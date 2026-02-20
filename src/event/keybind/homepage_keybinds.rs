use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::global_state::{Roudy, RoudyMessage};




pub async fn listen_for_homepage_binds(key: KeyEvent, global_state: &mut Roudy) {
    if key.code == KeyCode::Char('j') || key.code == KeyCode::Down {
        let new_offset = global_state.homepage_scroll_offset +1;
        Roudy::update(global_state, RoudyMessage::HOMEPAGEUpdateScrollOffset(new_offset));
    } else if key.code == KeyCode::Char('k') || key.code == KeyCode::Up {
        let new_offset = global_state.homepage_scroll_offset -1;
        Roudy::update(global_state, RoudyMessage::HOMEPAGEUpdateScrollOffset(new_offset));
    }
}