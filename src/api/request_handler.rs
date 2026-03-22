use crate::api::soundcloud::{playlists::{playlist::{APIPlaylist, get_playlists}, playlist_tracks::{APIPlaylistTracks, get_playlist_tracks}}, profile::{APIProfile, get_profile}, tracks::{track_hls_playlist::get_track_manifest, track_hls_segments::get_track_segments, track_metadata::track_metadata, track_urls::get_streaming_track_urls}};
use m3u8_rs::{MediaPlaylist, parse_media_playlist};
use tokio::sync::mpsc::{Receiver, Sender};
pub enum ClientEvent {
    GetProfile,
    GetPlaylists,
    GetPlaylistTrack(String),
    StreamTrack(String),
    GetTrackMetadata(u64),
    UpdateAccessToken(String),
    Shutdown,
}
pub enum ApiOutput {
    Profile(APIProfile),
    Playlists(Vec<APIPlaylist>),
    PlaylistTracks(Vec<APIPlaylistTracks>),
    TrackStream(Vec<u8>),
    TrackMediaPlaylist((Vec<u8>,MediaPlaylist)),
    TrackMetadata(String),
    Error(String),
}

pub struct ApiRequestHandler {
    pub api_req_handler_messenger: Sender<ClientEvent>,
    pub api_data_receiver: Receiver<ApiOutput>,
}

impl ApiRequestHandler {
    pub async fn mount(mut access_token: String) -> Self {
    let client = reqwest::Client::new();

    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<ClientEvent>(32);
    let (data_tx, data_rx) = tokio::sync::mpsc::channel::<ApiOutput>(32);

    tokio::spawn(async move {
        loop {
            if let Ok(event) = event_rx.try_recv() {
                match event {
                    ClientEvent::GetProfile => {
                        let profile_data = get_profile(&client, &access_token).await;
                        match profile_data {
                            Ok(data) => {
                                let _ = data_tx.send(ApiOutput::Profile(data)).await;
                            }
                            Err(e) => {
                                let _ = data_tx.send(ApiOutput::Error(e.to_string())).await;
                            }
                        }
                    }
                    ClientEvent::GetPlaylists => {
                        let playlist_data = get_playlists(&client, &access_token).await;
                        match playlist_data {
                            Ok(data) => {
                                let _ = data_tx.send(ApiOutput::Playlists(data)).await;
                            }
                            Err(e) => {
                                let _ = data_tx.send(ApiOutput::Error(e.to_string())).await;
                            }
                        }
                    }
                    ClientEvent::GetPlaylistTrack(playlist_urn) => {
                        let playlist_tracks = get_playlist_tracks(&client, &access_token, playlist_urn).await;
                        match playlist_tracks {
                            Ok(data) => {
                                let _ = data_tx.send(ApiOutput::PlaylistTracks(data)).await;
                            }
                            Err(e) => {
                                let _ = data_tx.send(ApiOutput::Error(e.to_string())).await;
                            }
                        }
                    }
                    ClientEvent::StreamTrack(id) => {
                        let stream = get_streaming_track_urls(&client, &access_token, id).await;
                        match stream {
                            Ok(data) => {
                                let manifest = get_track_manifest(&client, &access_token, data.hls_mp3_128_url).await;
                                match manifest {
                                    Ok(playlist) => {
                                        match parse_media_playlist(playlist.as_bytes()) {
                                            Ok(media_playlist) => {
                                                let mut uris = vec![];
                                                for segment in &media_playlist.1.segments {
                                                    let uri = &segment.uri;
                                                    uris.push(uri);
                                                }

                                                for url in uris {
                                                    let segments = get_track_segments(&client, &access_token, url.to_string()).await;
                                                    match segments {
                                                        Ok(bytes) => {
                                                            let _ = data_tx.send(ApiOutput::TrackStream(bytes.to_vec())).await;
                                                        }
                                                        Err(e) => {
                                                            let _ = data_tx.send(ApiOutput::Error(e.to_string())).await;
                                                        }
                                                    }
                                                }

                                                let _ = data_tx.send(ApiOutput::TrackMediaPlaylist((media_playlist.0.to_vec(), media_playlist.1))).await;
                                            }
                                            Err(e) => {
                                                let _ = data_tx.send(ApiOutput::Error(e.to_string())).await;
                                            }
                                        };
                                    }
                                    Err(e) => {
                                        let _ = data_tx.send(ApiOutput::Error(e.to_string())).await;
                                    }
                                };
                            }
                            Err(e) => {
                                let _ = data_tx.send(ApiOutput::Error(e.to_string())).await;
                            }
                        }
                    }
                    ClientEvent::GetTrackMetadata(id) => {
                        let meta_data = track_metadata(&client, &access_token, id).await;
                        match meta_data {
                            Ok(data) => {
                                let _ = data_tx.send(ApiOutput::TrackMetadata(data)).await;
                            }
                            Err(e) => {
                                let _ = data_tx.send(ApiOutput::Error(e.to_string())).await;
                            }
                        }
                    }
                    ClientEvent::UpdateAccessToken(token) => {
                        access_token = token;
                    }
                    ClientEvent::Shutdown => {
                        event_rx.close();
                        break
                    }
                };
            };
        }
    });
    Self {
        api_data_receiver: data_rx,
        api_req_handler_messenger: event_tx
    }
}

}


