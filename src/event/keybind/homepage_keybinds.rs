use ratatui::crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc::Sender;

use crate::{
    api::request_handler::ClientEvent,
    global_state::{ApiData, HomepageSubpage, Roudy, RoudyMessage},
};



pub async fn listen_for_homepage_binds(
    key: KeyEvent,
    req_api_data: &Option<Sender<ClientEvent>>,
    global_state: &mut Roudy,
    api_data: &mut ApiData,
) {
    match global_state.homepage_subpage {
        HomepageSubpage::AllPlaylists => {
            if key.code == KeyCode::Char('j') || key.code == KeyCode::Down {
                let potential_offset = global_state.homepage_playlist_scroll_offset + 1;
                let new_offset =
                    if potential_offset >= global_state.homepage_playlist_count as i32 - 1 {
                        0
                    } else {
                        potential_offset
                    };
                Roudy::update(
                    global_state,
                    RoudyMessage::HOMEPAGEUpdatePlaylistScrollOffset(new_offset),
                );
            } else if key.code == KeyCode::Char('k') || key.code == KeyCode::Up {
                let potential_offset = global_state.homepage_playlist_scroll_offset - 1;
                let new_offset = if potential_offset <= 0 {
                    (global_state.homepage_playlist_count - 1) as i32
                } else {
                    potential_offset
                };
                Roudy::update(
                    global_state,
                    RoudyMessage::HOMEPAGEUpdatePlaylistScrollOffset(new_offset),
                );
            } else if key.code == KeyCode::Enter {
                if let Some(sender) = req_api_data {
                    Roudy::update(global_state, RoudyMessage::HOMEPAGEChangeSubpage(HomepageSubpage::TracksInPlaylist));
                    if let Some(playlist_data) = api_data.playlists.as_ref() {

                        let uri = playlist_data[global_state.homepage_playlist_scroll_offset as usize]
                        .uri
                        .clone();
                        let uri_split = uri.rsplit(":").next();
                    
                        if let Some(id) = uri_split {
                            let _ = sender
                                .send(ClientEvent::GetPlaylistTrack(id.to_string()))
                                .await;
                        }   
                    }
                }
            }
        }
        HomepageSubpage::TracksInPlaylist => {
            if key.code == KeyCode::Esc {
                Roudy::update(global_state, RoudyMessage::HOMEPAGEChangeSubpage(HomepageSubpage::AllPlaylists));
            
            } else if key.code == KeyCode::Char('j') || key.code == KeyCode::Down {
                let potential_offset = global_state.homepage_tracks_scroll_offset + 1;
                let new_offset =
                    if potential_offset >= global_state.homepage_tracks_count as i32 - 1 {
                        0
                    } else {
                        potential_offset
                    };
                Roudy::update(
                    global_state,
                    RoudyMessage::HOMEPAGEUpdateTracksScrollOffset(new_offset),
                );
            } else if key.code == KeyCode::Char('k') || key.code == KeyCode::Up {
                let potential_offset = global_state.homepage_tracks_scroll_offset - 1;
                let new_offset = if potential_offset <= 0 {
                    (global_state.homepage_tracks_count - 1) as i32
                } else {
                    potential_offset
                };
                Roudy::update(
                    global_state,
                    RoudyMessage::HOMEPAGEUpdateTracksScrollOffset(new_offset),
                );
            } else if key.code == KeyCode::Enter {
                if let Some(sender) = req_api_data {
                    if let Some(tracks) = api_data.playlist_tracks.as_ref() {
                        let id = tracks[global_state.homepage_tracks_scroll_offset as usize].id;
                        let _ = sender.send(ClientEvent::StreamTrack(id)).await;
                    }
                }
            }
        }
        _ => {}
    }
}
