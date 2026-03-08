# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build          # Build the project
cargo run            # Run the application
cargo build --release && cargo run --release  # Production build
cargo check          # Fast type checking without building
```

## Architecture Overview

**Roudy** is a terminal UI application for SoundCloud, built with Ratatui. It uses OAuth2 authentication and an async event-driven architecture.

### Core Pattern: Multi-channel Message Passing

All components communicate via `tokio::sync::mpsc` channels. The main event loop (`event/eloop.rs`) orchestrates everything by polling all receivers and dispatching to handlers.

```
main.rs
  └→ eloop.rs (orchestrator)
       ├→ api/server.rs          (OAuth TCP callback server on :3231)
       ├→ api/request_handler.rs (API dispatcher, receives ClientEvent)
       ├→ credentials_manager.rs (Token lifecycle, reads/writes auth_credentials.json)
       ├→ keypress_polling.rs    (crossterm keyboard events)
       └→ layout/ui.rs           (terminal rendering)
```

### Key Message Types (`types.rs`)

| Type | Direction | Purpose |
|------|-----------|---------|
| `RoudyMessage` | → state | App state updates (Login, ChangeTab, scroll offset, etc.) |
| `ClientEvent` | → API handler | API requests (GetProfile, GetPlaylists, GetPlaylistTrack, UpdateAccessToken, Shutdown) |
| `ApiOutput` | ← API handler | API responses (Profile, Playlists, PlaylistTracks, Error) |
| `ServerEvent` | ← OAuth server | OAuth callback (Url, Shutdown) |
| `CredentialsEvent` | → creds manager | Token ops (SaveToken, Shutdown) |
| `CredentialsOutputEvent` | ← creds manager | Token status (AccessToken, Error, PromptLogin) |

### State Structures (`global_state.rs`)

- `Roudy` — UI state: `logged_in`, `selected_tab` (0=Home, 1=Profile, 2=Errors), `homepage_scroll_offset`, `homepage_playlist_count`
- `RoudyData` — Auth data: `login_url`
- `ApiData` — Cached API responses: `profile`, `playlists`, `playlist_tracks`
- `ErrorState` — Error flags and log strings

### Authentication Flow

1. App starts → `CredentialsManager` checks for `auth_credentials.json`
2. If missing/expired → display login page with OAuth URL
3. User completes OAuth → callback hits `localhost:3231/token?code=XXX&state=YYY`
4. CSRF token validated → code exchanged for access token → saved to `auth_credentials.json`
5. Main UI mounts and API requests begin

### UI Structure

Three-tab layout:
- **Home (Tab 0):** Paginated playlist list — name, duration, track count
- **Profile (Tab 1):** User profile info
- **Errors (Tab 2):** API/credentials error log

Keybinds: `Tab` switches pages, `q`/`Esc` quits, arrow keys scroll home page.

### API Layer (`api/soundcloud/`)

Each endpoint is its own module (`profile.rs`, `playlist.rs`, `playlist_tracks.rs`). They receive a `reqwest::Client` and return `ApiOutput`. The `request_handler.rs` runs as a long-lived async task receiving `ClientEvent` messages and calling the appropriate API module.

### Configuration

- `.env` — OAuth2 client credentials (ClientID, ClientSecret, redirect URI); required at runtime
- `auth_credentials.json` — Saved OAuth token (gitignored); written at runtime
