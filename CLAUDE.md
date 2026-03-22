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

All components communicate via `tokio::sync::mpsc` channels (async) and `std::sync::mpsc` (audio thread). The main event loop (`event/eloop.rs`) orchestrates everything in a synchronous loop that calls each listener sequentially per iteration.

```
main.rs
  └→ eloop.rs (orchestrator loop)
       ├→ credentials_output_listener.rs  (handles CredentialsOutputEvent)
       ├→ keypress_output_listener.rs     (handles PollEvent → dispatches to homepage_keybinds)
       ├→ auth_server_listener.rs         (handles ServerEvent from OAuth callback)
       ├→ api_output_listener.rs          (handles ApiOutput, updates global state, sends to audio)
       ├→ api/request_handler.rs          (API dispatcher, receives ClientEvent)
       ├→ auth_server/server.rs           (OAuth TCP callback server on :3231)
       ├→ credentials_manager.rs          (Token lifecycle, reads/writes auth_credentials.json)
       ├→ keypress_polling.rs             (crossterm keyboard events → PollEvent)
       ├→ audio/audio_handler.rs          (dedicated std::thread for audio playback)
       └→ layout/ui.rs                    (terminal rendering)
```

### Key Message Types

Message types are spread across multiple modules:

| Type | Location | Direction | Purpose |
|------|----------|-----------|---------|
| `RoudyMessage` | `global_state.rs` | → state | App state updates (Login, ChangeTab, subpage, scroll offsets, counts) |
| `RoudyDataMessage` | `global_state.rs` | → state | Auth data updates (SetLoginURL) |
| `ApiDataMessage` | `global_state.rs` | → state | Cache API responses (ProfileFetched, PlaylistsFetched, PlaylistTracksFetched, TrackStreamFetched, TrackMetadataFetched) |
| `ErrorMessage` | `global_state.rs` | → state | Error flag updates and log entries |
| `ClientEvent` | `request_handler.rs` | → API handler | API requests (GetProfile, GetPlaylists, GetPlaylistTrack, StreamTrack, GetTrackMetadata, UpdateAccessToken, Shutdown) |
| `ApiOutput` | `request_handler.rs` | ← API handler | API responses (Profile, Playlists, PlaylistTracks, TrackStream, TrackMediaPlaylist, TrackMetadata, Error) |
| `AudioCommand` | `audio_handler.rs` | → audio thread | Audio control (HlsReceived, Pause, Resume, Shutdown) |
| `AudioMessage` | `audio_handler.rs` | → AudioHandler | State updates (StoreMediaPlaylist) |
| `ServerEvent` | `types.rs` | ← OAuth server | OAuth callback (Url, Shutdown) |
| `PollEvent` | `types.rs` | ← keypress polling | Keyboard input (Input(KeyEvent)) |
| `CredentialsEvent` | `credentials_manager.rs` | → creds manager | Token ops (SaveToken, Shutdown) |
| `CredentialsOutputEvent` | `credentials_manager.rs` | ← creds manager | Token status (AccessToken, Error, PromptLogin) |

### State Structures (`global_state.rs`)

- `Roudy` — UI state: `logged_in`, `selected_tab` (enum: Home, Profile, ErrorStatus, Test), `homepage_playlist_scroll_offset`, `homepage_playlist_count`, `homepage_subpage` (enum: AllPlaylists, TracksInPlaylist), `homepage_tracks_scroll_offset`, `homepage_tracks_count`
- `RoudyData` — Auth data: `login_url`
- `ApiData` — Cached API responses: `profile`, `playlists`, `playlist_tracks`, `track_stream`, `track_metadata`
- `ErrorState` — Error flags (`failed_to_parse_code_param`, `csrf_token_does_not_match`, `failed_to_shutdown_server`, `failed_to_parse_csrf_param`, `failed_to_mount_api_request_handler`) and log vectors (`api_error_log`, `credentials_error_log`)

### Authentication Flow

1. App starts → `CredentialsManager` checks for `auth_credentials.json`
2. If missing/expired → display login page with OAuth URL (uses PKCE)
3. User completes OAuth → callback hits `localhost:3231/token?code=XXX&state=YYY`
4. CSRF token validated → code exchanged for access token → saved to `auth_credentials.json`
5. If token exists but expired → refresh via refresh token
6. `ApiRequestHandler` mounted with access token → main UI renders and API requests begin

### UI Structure

Four-tab layout:
- **Home:** Two-level subpage navigation:
  - AllPlaylists — Playlist carousel: name, duration, track count
  - TracksInPlaylist — Playlist tracks carousel: track listing for selected playlist
- **Profile:** User profile info
- **ErrorStatus:** API/credentials error log
- **Test:** Debug page for raw track metadata JSON

Keybinds:
- `Tab` — switch between tabs
- `q` — quit application
- `j`/`Down` — scroll down in current list
- `k`/`Up` — scroll up in current list
- `Enter` — select playlist (AllPlaylists→TracksInPlaylist) or stream track via HLS (TracksInPlaylist)
- `Esc` — go back from playlist tracks to playlists (subpage 1→0)

### Audio Pipeline (`audio/`)

- `audio_handler.rs` — Dedicated `std::thread` receiving `AudioCommand` messages via `std::sync::mpsc`. On first HLS segment: creates cpal device, ring buffer (SPSC via ringbuf), and output stream. Subsequent segments are decoded and pushed into the existing ring buffer. Holds the cpal `Stream` to keep playback alive.
- `decoder.rs` — Decodes MP3 bytes (`Vec<u8>`) to PCM `Vec<f32>` using Symphonia. Returns `DecodedStream` with samples, sample_rate, and channel count.
- `player.rs` — Legacy standalone player (replaced by HLS handler in audio_handler.rs). Builds cpal output stream for a single `Vec<f32>`.

### API Layer (`api/soundcloud/`)

Endpoints are organized into submodules:
- `profile.rs` — GET /me
- `playlists/playlist.rs` — GET /me/playlists
- `playlists/playlist_tracks.rs` — GET /playlists/{id}/tracks
- `tracks/track_urls.rs` — GET /tracks/{id}/streams → `TrackUrls` struct (http_mp3_128, hls_mp3_128, hls_aac_160, preview_mp3_128)
- `tracks/track_hls_playlist.rs` — Fetches M3U8 manifest from HLS URL
- `tracks/track_hls_segments.rs` — Fetches individual HLS audio segment bytes
- `tracks/track_metadata.rs` — GET /tracks/{id} → raw JSON
- `auth_client.rs` — OAuth2 PKCE login/token exchange

The `request_handler.rs` runs as a long-lived async task receiving `ClientEvent` messages. For `StreamTrack`: fetches stream URLs → fetches HLS manifest → parses M3U8 → downloads each segment and sends bytes to the audio thread via `api_output_listener`.

### Configuration

- `.env` — OAuth2 client credentials (CLIENT_ID, CLIENT_SECRET, REDIRECT_URI); required at runtime
- `auth_credentials.json` — Saved OAuth token with refresh_token (gitignored); written at runtime
