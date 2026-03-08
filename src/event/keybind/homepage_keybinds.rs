use ratatui::crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc::Sender;

use crate::{api::request_handler::ClientEvent, global_state::{ApiData, Roudy, RoudyMessage}};




pub async fn listen_for_homepage_binds(key: KeyEvent, req_api_data: &Option<Sender<ClientEvent>>, global_state: &mut Roudy, api_data: &mut ApiData) {
    if key.code == KeyCode::Char('j') || key.code == KeyCode::Down {
        let new_offset = global_state.homepage_scroll_offset +1;
        Roudy::update(global_state, RoudyMessage::HOMEPAGEUpdateScrollOffset(new_offset));
    } else if key.code == KeyCode::Char('k') || key.code == KeyCode::Up {
        let new_offset = global_state.homepage_scroll_offset -1;
        Roudy::update(global_state, RoudyMessage::HOMEPAGEUpdateScrollOffset(new_offset));
    } else if key.code == KeyCode::Enter {
        if let Some(sender) = req_api_data {
            let playlist_urn = api_data.playlists.as_ref().expect("should have")[global_state.homepage_scroll_offset as usize].playlist_urn.clone();
            let _ = sender.send(ClientEvent::GetPlaylistTrack(playlist_urn)).await;
        }
    }
}