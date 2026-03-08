# The complete guide to cpal and audio playback for a Rust TUI player

**cpal is the foundational Rust crate for cross-platform audio I/O**, and for your "roudy" SoundCloud player, the optimal architecture pairs cpal directly with Symphonia for decoding — not Rodio — because SoundCloud's HLS streaming model demands fine-grained control over network buffering, non-seekable sources, and segment-level seeking that Rodio's abstractions cannot provide. cpal v0.17.3 (released February 2026) introduces stable device IDs, improved ALSA behavior, and buffer underrun reporting, making it the most capable version yet. On Linux, ALSA is the default backend, but PipeWire and PulseAudio native backends are arriving, and the interaction between these sound servers and ALSA is the single biggest gotcha you'll face.

This guide covers the full stack: from cpal's architecture down to writing samples in the audio callback, through Linux backend selection, and up to the complete decoding pipeline from SoundCloud's AAC HLS streams to PCM output.

---

## How cpal models audio: hosts, devices, and streams

cpal organizes audio around three core abstractions accessed through three corresponding traits in `cpal::traits`:

**Host** represents an OS audio backend. On Linux, the default host is ALSA. Optional hosts include JACK (`jack` feature), PipeWire (`pipewire` feature), and PulseAudio (`pulseaudio` feature). You get the default with `cpal::default_host()`, or enumerate all compiled-in backends with `cpal::available_hosts()` and instantiate a specific one with `cpal::host_from_id(id)`. Backend selection is **compile-time** via Cargo features, but when multiple backends are compiled in, you choose between them at **runtime**.

**Device** represents a physical or virtual audio endpoint. A Host provides `default_output_device()`, `default_input_device()`, or you can enumerate all with `devices()` / `output_devices()`. In v0.17, devices gained `id()` for stable identification (persist this across sessions) and `description()` for human-readable display, replacing the deprecated `name()`.

**Stream** is the active audio data flow. Output streams deliver PCM to the device; input streams capture it. Streams are created via `device.build_output_stream()` with a callback closure, and managed with `stream.play()` and `stream.pause()`. The `Stream` struct must be kept alive — **dropping it silently stops audio**, which is the single most common cpal footgun.

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

let host = cpal::default_host();
let device = host.default_output_device().expect("no output device");
let config = device.default_output_config()?;
println!("Output: {:?}, format: {:?}, rate: {}Hz, {}ch",
    device.description()?, config.sample_format(),
    config.sample_rate(), config.channels());
```

Configuration flows through `SupportedStreamConfigRange` (what the device can do) and `StreamConfig` (what you request). A `SupportedStreamConfigRange` reports min/max sample rates, channel count, sample format, and supported buffer sizes. Call `.with_max_sample_rate()` to collapse it into a concrete `SupportedStreamConfig`, then `.into()` to get a `StreamConfig` for stream construction.

---

## Setting up cpal on Linux and understanding feature flags

The minimum dependency is straightforward:

```toml
[dependencies]
cpal = "0.17"
```

On Linux, **ALSA development files are always required**, even when targeting JACK or PipeWire:

| Backend | Debian/Ubuntu | Fedora |
|---|---|---|
| ALSA (always required) | `libasound2-dev` | `alsa-lib-devel` |
| JACK (optional) | `libjack-jackd2-dev` | `jack-devel` |
| PipeWire (optional) | `libpipewire-0.3-dev` | `pipewire-devel` |
| PulseAudio (optional) | `libpulse-dev` | `pulseaudio-libs-devel` |

The key feature flags for Linux are:

- **`jack`** — Enables the JACK backend. PipeWire's JACK compatibility layer also works here.
- **`pipewire`** — Native PipeWire backend via `pipewire-rs`. Added via PR #938; may require building from git if not yet released in 0.17.3.
- **`pulseaudio`** — Native PulseAudio backend. Added via PR #957; same release caveat applies.
- **`realtime_audio_thread`** — Promotes the audio callback thread to real-time priority using `audio_thread_priority`. On Linux, this requires `rtkit` or appropriate `/etc/security/limits.conf` entries.
- **`custom_host`** — Allows user-defined Host/Device/Stream implementations.

The MSRV for the ALSA backend is **Rust 1.82** due to the `alsa-sys` dependency. cpal itself declares `rust-version = "1.70"` in Cargo.toml, but the actual minimum depends on your target backend.

---

## Building an output stream and the callback model

The heart of cpal is `build_output_stream`, which creates a stream that calls your closure on a dedicated audio thread whenever the OS needs more samples:

```rust
fn build_output_stream<T, D, E>(
    &self,
    config: &StreamConfig,
    data_callback: D,
    error_callback: E,
    timeout: Option<Duration>,
) -> Result<Stream, BuildStreamError>
where
    T: SizedSample,
    D: FnMut(&mut [T], &OutputCallbackInfo) + Send + 'static,
    E: FnMut(StreamError) + Send + 'static;
```

The type parameter `T` determines the sample format — typically `f32`, `i16`, or `u16`. The data callback receives a mutable slice of interleaved samples. For stereo, the layout is `[L0, R0, L1, R1, ...]`. A **frame** is one sample per channel, so the buffer contains `buffer_length / channels` frames. You iterate by frame using `data.chunks_mut(channels)`.

Here's a complete playback pattern that generates a 440Hz sine wave:

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device");
    let supported = device.default_output_config()?;
    let sample_rate = supported.sample_rate() as f32;
    let channels = supported.channels() as usize;
    let config: cpal::StreamConfig = supported.into();

    let mut phase = 0f32;
    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let sample = (phase * 2.0 * std::f32::consts::PI).sin() * 0.2;
                phase = (phase + 440.0 / sample_rate) % 1.0;
                for s in frame.iter_mut() {
                    *s = sample;
                }
            }
        },
        |err| eprintln!("stream error: {err}"),
        None,
    )?;

    stream.play()?;
    std::thread::sleep(std::time::Duration::from_secs(3));
    Ok(())
}
```

When the device's default format isn't `f32`, match on `sample_format()` and use `FromSample` for conversion:

```rust
match supported.sample_format() {
    cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config),
    cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config),
    cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config),
    fmt => panic!("unsupported format: {fmt}"),
}
```

cpal performs **no automatic sample rate conversion or format conversion** — what you configure is exactly what hits the hardware. The `SampleFormat` enum in v0.17 is `#[non_exhaustive]` and includes **I8, I16, I24, I32, I64, U8, U16, U24, U32, U64, F32, F64**, plus DSD variants. The most common output formats on Linux are **F32** and **I16**. The `Sample` trait (from `dasp_sample`) provides `EQUILIBRIUM` as the silence value for any format.

---

## Threading, state sharing, and the rules of the audio callback

**The audio callback runs on a separate, high-priority thread.** On modern backends, the OS provides this thread. On ALSA (which has a blocking API), cpal spawns its own thread per stream. The callback must satisfy `FnMut + Send + 'static`.

The cardinal rule: **never block inside the audio callback**. The callback deadline is `buffer_size / sample_rate` seconds (e.g., 512 frames at 48kHz = ~10.7ms). Violating this causes audible glitches. Specifically, avoid:

- Memory allocations (`Vec::push`, `Box::new`, `String` operations)
- Mutex locks that could contend with other threads
- File or network I/O
- `println!` or any logging that writes to stdout/stderr
- Unbounded computation

For "roudy", you need to feed decoded audio from a network/decode thread into the audio callback. The recommended patterns, ranked by suitability:

**Lock-free ring buffer (best for audio data)** — The `ringbuf` crate provides SPSC (single-producer, single-consumer) lock-free queues. cpal's own examples use this:

```rust
use ringbuf::{traits::*, HeapRb};

let rb = HeapRb::<f32>::new(4096 * 4); // generous sizing
let (mut producer, mut consumer) = rb.split();

// Decode thread pushes samples:
for sample in decoded_samples {
    let _ = producer.try_push(sample); // non-blocking, drops if full
}

// Audio callback pulls samples:
let stream = device.build_output_stream(&config,
    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        for sample in data.iter_mut() {
            *sample = consumer.try_pop().unwrap_or(0.0); // silence on underrun
        }
    },
    |err| eprintln!("error: {err}"),
    None,
)?;
```

**Atomics (best for control parameters)** — Use `AtomicBool` for pause flags and `AtomicU32` (with `f32::to_bits`/`from_bits`) for volume:

```rust
use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use std::sync::Arc;

let volume = Arc::new(AtomicU32::new(f32::to_bits(1.0)));
let paused = Arc::new(AtomicBool::new(false));

let vol = volume.clone();
let p = paused.clone();
// In callback:
move |data: &mut [f32], _| {
    let v = f32::from_bits(vol.load(Ordering::Relaxed));
    if p.load(Ordering::Relaxed) {
        data.fill(0.0);
        return;
    }
    for sample in data.iter_mut() {
        *sample = consumer.try_pop().unwrap_or(0.0) * v;
    }
}
```

Avoid `Arc<Mutex<>>` in the output callback. It's acceptable for input recording (where occasional blocking is tolerable) but risks priority inversion on the output path.

---

## Error handling and stream lifecycle

cpal's error types are well-structured. **`BuildStreamError`** has variants for `DeviceNotAvailable`, `StreamConfigNotSupported`, `InvalidArgument`, and `BackendSpecific`. **`StreamError`** (delivered to the error callback at runtime) includes `DeviceNotAvailable`, `BackendSpecific`, and two v0.17 additions: **`BufferUnderrun`** (ALSA, JACK, CoreAudio now report these) and **`StreamInvalidated`** (stream must be rebuilt, e.g., JACK server sample rate change).

On Linux, the most common errors are:

- **`DeviceBusy`** — PipeWire or PulseAudio holds the ALSA default device exclusively. Solution: use bridge devices or native backend features.
- **Buffer underruns on stream start** — Known issue (#460); ALSA's `PCM.try_recover()` handles recovery automatically in v0.17.
- **`StreamConfigNotSupported`** — Requesting unsupported sample rates or buffer sizes.

A critical subtlety: **`stream.play()` and `stream.pause()` may not return errors immediately on Linux/Windows**. These backends enqueue commands; errors arrive later through the error callback. Only macOS returns errors synchronously.

For stream lifecycle, the essential pattern for "roudy":

```rust
struct AudioPlayer {
    stream: cpal::Stream,           // MUST be kept alive
    producer: ringbuf::HeapProd<f32>, // feed samples here
    volume: Arc<AtomicU32>,
    paused: Arc<AtomicBool>,
}

impl AudioPlayer {
    fn pause(&self)  { self.paused.store(true, Ordering::Relaxed); }
    fn resume(&self) { self.paused.store(false, Ordering::Relaxed); }
    fn set_volume(&self, v: f32) {
        self.volume.store(f32::to_bits(v.clamp(0.0, 1.0)), Ordering::Relaxed);
    }
}
```

---

## Buffer size, latency, and Linux backend selection

Buffer size directly controls latency. `StreamConfig` accepts a `BufferSize` enum: `Default` or `Fixed(FrameCount)`. In v0.17, **`BufferSize::Default` changed behavior** — it now defers to the device/host default instead of cpal's old opinionated ~100ms default. On ALSA, this can range from PipeWire's quantum (~1024 frames ≈ 21ms at 48kHz) to `u32::MAX` on misconfigured hardware. **Always use `BufferSize::Fixed(n)` for predictable latency.** Query the supported range first:

```rust
if let cpal::SupportedBufferSize::Range { min, max } = supported.buffer_size() {
    let desired = 512u32.clamp(*min, *max);
    config.buffer_size = cpal::BufferSize::Fixed(desired);
}
```

At **512 frames / 48kHz**, the callback fires every ~10.7ms. Smaller buffers mean lower latency but higher CPU load and underrun risk. For a music player (not a live instrument), **1024–2048 frames** is a comfortable sweet spot.

The Linux backend situation deserves careful attention. Most modern Linux desktops run **PipeWire**, which provides compatibility layers for both PulseAudio and ALSA. When PipeWire is running, it holds the ALSA `default` device exclusively. This means cpal's default ALSA backend can fail with `DeviceBusy` if another application already has a stream open. Three solutions exist:

1. **Use ALSA bridge devices** — Instead of `default`, connect to the `pipewire` or `pulse` ALSA device names, which route through the sound server.
2. **Use native PipeWire/PulseAudio features** — Compile with `--features pipewire` or `--features pulseaudio` for direct server integration. This is the cleanest approach but these features may still be on cpal's master branch, not yet in a release.
3. **Use JACK** — PipeWire implements the JACK API, so `--features jack` works through PipeWire's JACK compatibility layer.

For "roudy", the pragmatic choice today is: **target ALSA as default** (it works through PipeWire's ALSA compatibility on most systems), add the `jack` feature as an option, and adopt native PipeWire/PulseAudio features once they're in a stable cpal release. Users may need to be in the `audio` group if their system doesn't grant audio access via logind.

---

## Decoding SoundCloud streams: Symphonia + cpal, not Rodio

SoundCloud **migrated to AAC-based HLS streams** in late 2025, deprecating MP3 progressive streams and Opus HLS. The current formats are:

- **`hls_aac_160_url`** — AAC 160kbps in HLS (primary)
- **`hls_aac_96_url`** — AAC 96kbps in HLS (lower quality)
- **`preview_mp3_128_url`** — 30-second MP3 preview clips

This means your decoder must handle **AAC in MPEG-TS or fMP4 containers** served via HLS playlists. The decoding stack recommendation:

**Symphonia** (v0.5.5, pure Rust, MPL-2.0) is the right choice. It supports AAC-LC, MP3, Vorbis, FLAC, and ALAC, with container support for ISO/MP4, OGG, MKV, and ADTS. Critically, it handles non-seekable sources via `ReadOnlySource` — essential for network streams. It does **not** support Opus, but SoundCloud deprecated Opus streams, so this gap is irrelevant for your use case.

```toml
symphonia = { version = "0.5.5", features = ["aac", "isomp4", "mp3", "adts"] }
```

**Rodio** (v0.22, built on cpal) seems tempting — it provides automatic sample rate conversion, volume control, and mixing. But it has a critical limitation: `Decoder::new()` requires `Read + Seek`, and **panics on non-seekable sources**. Since HLS segments arrive over HTTP, you'd need to buffer entire segments to `Cursor<Vec<u8>>` before decoding, losing Rodio's simplicity advantage. Rodio is appropriate for simple file playback apps but not for a streaming player.

The Symphonia decode loop extracts interleaved PCM via `SampleBuffer`:

```rust
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, ReadOnlySource};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

// Wrap an HTTP response body (implements Read)
let source = ReadOnlySource::new(segment_reader);
let mss = MediaSourceStream::new(Box::new(source), Default::default());

let mut hint = Hint::new();
hint.mime_type("audio/aac");

let probed = symphonia::default::get_probe()
    .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())?;

let mut format = probed.format;
let track = format.tracks().iter()
    .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
    .expect("no audio track");
let track_id = track.id;

let mut decoder = symphonia::default::get_codecs()
    .make(&track.codec_params, &DecoderOptions::default())?;

let mut sample_buf = None;
loop {
    match format.next_packet() {
        Ok(packet) if packet.track_id() == track_id => {
            let decoded = decoder.decode(&packet)?;
            let buf = sample_buf.get_or_insert_with(|| {
                SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec())
            });
            buf.copy_interleaved_ref(decoded);
            // buf.samples() is now &[f32] — interleaved, ready for cpal
            for &s in buf.samples() {
                let _ = producer.try_push(s);
            }
        }
        Err(symphonia::core::errors::Error::IoError(ref e))
            if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
        Err(e) => return Err(e.into()),
        _ => continue,
    }
}
```

**Sample rate conversion** is essential — SoundCloud audio is typically 44100Hz, but your output device may default to 48000Hz. Use the **rubato** crate (pure Rust, allocation-free during processing):

```toml
rubato = "0.17"
```

Rubato works with non-interleaved channel vectors, so you'll de-interleave Symphonia's output, resample, then re-interleave for cpal.

---

## The full architecture for roudy

The recommended thread architecture for a TUI SoundCloud player:

```
 ┌──────────┐    ┌───────────────┐    ┌──────────────┐    ┌──────────┐
 │ Network  │───→│ Decode/Resamp │───→│  Ring Buffer  │───→│  cpal    │
 │ (tokio)  │    │ (Symphonia +  │    │  (ringbuf)    │    │ callback │
 │ HLS fetch│    │  rubato)      │    │               │    │ thread   │
 └──────────┘    └───────────────┘    └──────────────┘    └──────────┘
       ↑                                                        ↑
       │              ┌──────────────┐                          │
       └──────────────│  UI thread   │──── atomics/commands ────┘
                      │  (ratatui)   │
                      └──────────────┘
```

**Network thread** (async, tokio + reqwest): Fetches the HLS M3U8 playlist from SoundCloud's `/tracks/{id}/streams` endpoint, parses it with `m3u8-rs`, and downloads segments sequentially. Stream URLs include CloudFront signed parameters with **short expiry times** — implement proactive re-fetching.

**Decode thread**: Receives segment data, decodes with Symphonia, resamples with rubato if needed, and pushes interleaved f32 PCM into the ring buffer. Pre-buffer **2–3 segments** before starting playback.

**Audio callback thread** (managed by cpal): Pulls samples from the ring buffer. Falls back to silence (0.0) on underrun. Applies volume scaling via atomic.

**UI thread** (main, ratatui + crossterm): Renders the TUI, handles keyboard input, sends commands (play/pause/seek/volume) via atomics and channels.

For **seeking in HLS**, calculate which segment contains the target timestamp using `#EXTINF` durations from the playlist, skip to that segment, and use Symphonia's `format.seek()` for intra-segment positioning. For **gapless playback** between segments, enable `FormatOptions::enable_gapless = true` and keep the decoder alive across segment boundaries when the codec parameters match.

The recommended `Cargo.toml` for the full stack:

```toml
[dependencies]
cpal = "0.17"
symphonia = { version = "0.5.5", features = ["aac", "isomp4", "mp3", "adts"] }
rubato = "0.17"
m3u8-rs = "6"
ringbuf = "0.4"
reqwest = { version = "0.12", features = ["stream"] }
tokio = { version = "1", features = ["full"] }
ratatui = "0.29"
crossterm = "0.28"
anyhow = "1"
```

---

## Version history and migration notes

cpal **v0.17.3** (February 18, 2026) is current. The v0.16 → v0.17 migration introduced several breaking changes worth noting:

**`SampleRate` changed from a newtype struct to a plain `u32` type alias.** Code using `SampleRate(44100)` must change to just `44100`. **`Device::name()` is deprecated** in favor of `id()` (stable, persistable) and `description()` (human-readable). **`BufferSize::Default` behavior changed** to defer to host/device defaults rather than cpal's opinionated values — if you relied on the old behavior, switch to `BufferSize::Fixed(n)`. ALSA device enumeration now returns all `aplay -L` devices (v0.16 had a regression returning only card names). ALSA uses `set_buffer_size_near()` instead of exact `set_buffer_size()`, improving hardware compatibility.

The v0.13 release (2020) was the largest architectural shift, removing the blocking `EventLoop` API entirely in favor of the current callback model. Any pre-0.13 examples or tutorials are fundamentally incompatible with the modern API.

## Conclusion

For "roudy", the architecture is clear: **cpal v0.17 + Symphonia + rubato + ringbuf** gives you full control over the decode-to-output pipeline that a streaming music player demands. Rodio's convenience isn't worth its streaming limitations. The most impactful decisions you'll make are: sizing your ring buffer generously (4× the expected callback buffer), using `BufferSize::Fixed` rather than `Default` on Linux, and handling SoundCloud's URL expiration gracefully. The `termusic` project (a production Rust TUI music player using Symphonia + cpal) is worth studying as an architectural reference. Start with the sine-wave example to verify your audio stack works, then layer in Symphonia decoding segment-by-segment — the complexity is manageable when each layer is tested independently.