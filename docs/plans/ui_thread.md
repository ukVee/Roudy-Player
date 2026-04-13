# Plan: Move UI Rendering to a Dedicated Thread

## Context

You want smooth animations at a configurable frame rate (30/60/120 FPS). Currently, `terminal.draw()` runs inline in the `tokio::select!` loop in `eloop.rs` — it only redraws after an event arrives, so there's no consistent frame rate. Moving the UI to its own `std::thread` with a tick-based loop gives you independent, steady rendering.

The UI thread will be "dumb" — it only reads shared state and renders. All state mutation stays in the event loop.

---

## Teaching Topics (in order)

These are concepts you need before implementing. Each builds on the previous.

### 1. `Arc<Mutex<T>>` — Shared ownership with exclusive access

You know `Arc<AtomicBool>` — `Arc` for shared ownership, `AtomicBool` for lock-free thread-safe access to a single primitive. But structs like `Roudy` (7 fields, mixed types) can't be atomic. You need a **lock**.

`Mutex<T>` enforces: only one thread accesses `T` at a time.

- `.lock()` returns a `MutexGuard<T>` — gives `&mut T` via `DerefMut`
- If another thread holds the lock, your thread **blocks** (sleeps) until they drop the guard
- Guard auto-releases when it goes out of scope

**Why not use Mutex here:** Even two readers block each other. Your UI thread reads at 120 FPS and the event loop writes occasionally — they'd constantly fight for the lock unnecessarily.

### 2. `Arc<RwLock<T>>` — Multiple readers, one exclusive writer

`RwLock<T>` has two modes:

- `.read()` -> `RwLockReadGuard<T>` -> gives `&T`. **Multiple readers can hold read locks simultaneously.**
- `.write()` -> `RwLockWriteGuard<T>` -> gives `&mut T`. **Exclusive — no other readers or writers.**

This fits perfectly: UI thread only ever needs `&Roudy` (reads), event loop is the sole writer.

### 3. Mutex poisoning

Both `Mutex` and `RwLock` lock methods return `Result`, not the guard directly. The `Err` case is **poisoning**.

Poisoning happens when a thread **panics while holding a lock**. The lock gets marked "poisoned" because the data inside might be half-updated. All subsequent `.lock()`/`.read()`/`.write()` calls return `Err(PoisonError)`.

**How to handle:** For Roudy, if the event loop panics, the app is dead anyway. Using `.unwrap()` (or `.expect("msg")`) is standard — it means "if poisoned, panic this thread too." The panic hook in `main.rs` will catch it and restore the terminal.

### 4. Lock contention and hold time

**Contention** = multiple threads competing for the same lock, causing some to block.

**Key rule: hold locks for the minimum time necessary.**

```rust
// GOOD: lock only during draw, sleep with NO locks held
{
    let state = global_state.read().unwrap();
    terminal.draw(|f| ui(f, &state, ...)).unwrap();
} // guard dropped here
std::thread::sleep(tick_duration); // no lock held

// BAD: lock held during sleep — blocks event loop writes for entire tick
let state = global_state.read().unwrap();
terminal.draw(|f| ui(f, &state, ...)).unwrap();
std::thread::sleep(tick_duration);
// guard dropped here (way too late)
```

In Roudy, contention is naturally low (one reader at ~120 Hz, one sporadic writer), but minimizing hold time is still good practice.

### 5. RwLock reader/writer semantics (deeper)

Three behaviors to know:

- **Write starvation prevention:** When a writer calls `.write()` while readers hold `.read()`, most implementations block new readers from acquiring. This prevents an endless stream of readers from starving the writer.
- **Deadlock from same-thread re-locking:** If one thread holds `.read()` and tries `.write()` on the same RwLock (or vice versa), it deadlocks. Not a concern here — UI thread only reads, event loop only writes.
- **Lock ordering:** When acquiring locks on multiple RwLocks, always use the same order everywhere to prevent deadlock. Recommended order: `global_state` -> `roudy_data` -> `api_data` -> `error_state`.

### 6. The dirty flag pattern

At 120 FPS, most ticks have nothing new to render. A **dirty flag** (`Arc<AtomicBool>`) avoids wasted renders:

- Event loop: after any state mutation -> `dirty.store(true, Relaxed)`
- UI thread: check `dirty.load(Relaxed)` -> if `false`, skip render. If `true`, render then `dirty.store(false, Relaxed)`.

You already use this exact pattern with `paused: Arc<AtomicBool>`. Same idea.

`Ordering::Relaxed` is fine here because the `RwLock` itself provides the memory ordering guarantees. The dirty flag is just a hint — if one frame is skipped due to a stale read, the next frame catches it (~8ms later at 120 FPS, imperceptible).

### 7. `std::sync::RwLockWriteGuard` is `!Send` — the async complication

**This is critical.** `std::sync::RwLockWriteGuard` cannot be held across `.await` points in a tokio task. The compiler enforces this.

Two of your listeners are `async`:

- `credentials_listener` — calls `login_to_sc().await` while holding `&mut roudy_data`
- `keypress_listener` — calls `sender.send(...).await` while holding `&mut global_state`, `&mut api_data`

**Solution:** Restructure the event loop so write locks are acquired only around synchronous state mutations, and dropped before any `.await`. Pattern:

```rust
// Instead of: acquire lock -> call async listener -> drop lock
// Do: call async listener with owned/cloned data -> acquire lock -> apply mutations -> drop lock
```

The specific restructuring varies per branch (detailed in implementation steps below).

---

## Implementation Steps

### Step 1: Wrap state structs in `Arc<RwLock<T>>`

**File:** `src/event/eloop.rs`

Change:

```rust
let mut global_state = Roudy::new();
let mut roudy_data = RoudyData::new((...));
let mut error_state = ErrorState::new();
let mut api_data = ApiData::new();
```

To `Arc<RwLock<T>>` versions. Add `use std::sync::{Arc, RwLock};`.

This will break everything downstream — that's expected. You'll fix it in the next steps.

### Step 2: Restructure the event loop branches to handle the async/lock conflict

Each `tokio::select!` branch needs restructuring so write locks aren't held across `.await` points.

**Credentials branch** (currently line 50-56):

- `credentials_listener` calls `login_to_sc().await` only in the `PromptLogin` arm
- Split approach: do the async work first, then return what needs to be mutated, then acquire locks and apply mutations in `eloop.rs`
- Alternatively: pass owned/cloned data to the async parts, acquire locks only for the mutation phase

**Keypress branch** (currently line 57-73):

- State mutations (tab changes, scroll updates) happen first, then `.await` sends API requests
- Split into: acquire write locks -> mutate state -> drop locks -> then `.await` the sends
- The listener already does this conceptually — you may need to restructure it to return "what API requests to send" separately from "what state to update"

**Auth server branch** (currently line 75-79):

- Only mutates `error_state`. Check if `auth_server_listener` has `.await` calls that happen while holding state references.

**API output branch** (currently line 81-88):

- `api_listener` is synchronous (`fn`, not `async fn`) — no `.await` issue. Just acquire write locks, call it, drop locks.

### Step 3: Create dirty flag and shutdown flag

**File:** `src/event/eloop.rs`

Create two `Arc<AtomicBool>` values before the main loop:

- `dirty` — initialized to `true` (so the first frame renders)
- `ui_shutdown` — initialized to `false`

After every state mutation in each branch, set `dirty.store(true, Relaxed)`.
In the shutdown path (when `q` is pressed), set `ui_shutdown.store(true, Relaxed)`.

### Step 4: Create the UI thread function

**New file:** `src/layout/ui_thread.rs`

Function that takes:

- `Terminal<CrosstermBackend<Stdout>>` (ownership)
- `Arc<RwLock<...>>` clones for all 4 state structs
- `Arc<AtomicBool>` for dirty flag and shutdown flag
- `u32` for FPS

Inside: a loop that:

1. Checks shutdown flag -> break if true
2. Checks dirty flag -> if true, acquire read locks on all 4 structs, call `terminal.draw()`, drop locks
3. Sleeps for the remaining tick duration
4. Returns the terminal when done

Use `Instant`-based timing to compensate for render time:

```rust
let tick_duration = Duration::from_secs_f64(1.0 / fps as f64);
let mut next_tick = Instant::now() + tick_duration;

loop {
    // ... check shutdown, check dirty, render ...

    let now = Instant::now();
    if now < next_tick {
        std::thread::sleep(next_tick - now);
    }
    next_tick += tick_duration;
}
```

### Step 5: Update `event_loop()` signature

**File:** `src/event/eloop.rs`

`event_loop()` no longer owns the terminal. New signature:

```rust
pub async fn event_loop(
    global_state: Arc<RwLock<Roudy>>,
    roudy_data: Arc<RwLock<RoudyData>>,
    api_data: Arc<RwLock<ApiData>>,
    error_state: Arc<RwLock<ErrorState>>,
    dirty: Arc<AtomicBool>,
    ui_shutdown: Arc<AtomicBool>,
) -> anyhow::Result<()>
```

Remove `terminal.draw()` call (currently line 109-111). Remove terminal from return type.

### Step 6: Handle `RoudyData` creation timing

`AudioHandler::mount()` is currently called inside `event_loop()` (line 30), and its atomics are passed to `RoudyData`. But now `RoudyData` is created in `main.rs` (before `event_loop` runs).

Two options:

- **Option A:** Create `RoudyData` with default atomics in `main.rs`, then swap in real ones via write lock inside `event_loop()` after `AudioHandler::mount()`
- **Option B:** Move `AudioHandler::mount()` to `main.rs` and pass the handler into `event_loop()`

Option A is simpler — the UI shows default values (paused=false, volume=1.0) for the first few frames before the event loop starts, which matches current startup behavior.

### Step 7: Update `main.rs` to orchestrate everything

**File:** `src/main.rs`

New flow:

1. Terminal setup (raw mode, alternate screen) — unchanged
2. Create `Terminal` — unchanged
3. Create all 4 `Arc<RwLock<T>>` state structs
4. Create dirty and shutdown flags
5. Spawn UI thread (passing terminal + Arc clones + flags) -> get `JoinHandle`
6. Call `event_loop()` (passing Arc clones + flags, NO terminal)
7. After `event_loop()` returns, `ui_thread_handle.join().unwrap()` to get terminal back
8. Restore terminal — unchanged

### Step 8: Make FPS configurable

Start with a constant (`const DEFAULT_FPS: u32 = 120;`). Pass it to the UI thread spawn. Later you can read from a config file or CLI arg.

---

## Key files to modify

| File | Changes |
|------|---------|
| `src/main.rs` | Create shared state + flags, spawn UI thread, join on shutdown |
| `src/event/eloop.rs` | Accept `Arc<RwLock<T>>` params, acquire/release write locks per branch, set dirty flag, remove `terminal.draw()` |
| `src/layout/ui_thread.rs` | **New file** — tick loop with dirty flag check, read locks, `terminal.draw()` |
| `src/layout/mod.rs` | Add `pub mod ui_thread;` |
| `src/event/credentials_output_listener.rs` | May need restructuring to separate async work from state mutation |
| `src/event/keybind/keypress_output_listener.rs` | May need restructuring to separate state mutation from async sends |

Files that should NOT change:

- `src/layout/ui.rs` — `ui()` still takes `&Roudy`, `&RoudyData`, `&ApiData`, `&ErrorState`
- `src/global_state.rs` — state structs and `update()` methods stay the same
- `src/audio/audio_handler.rs` — audio thread is independent of this change

---

## Verification

1. `cargo check` after each step to catch type errors incrementally
2. `cargo run` — app should behave identically to before (same UI, same keybinds)
3. Verify smooth rendering by adding a simple animation (e.g. a counter or blinking element) that changes every frame — it should update at the configured FPS
4. Verify the dirty flag works: when idle (no keypresses, no API activity), CPU usage should be near zero despite the 120 FPS tick loop
