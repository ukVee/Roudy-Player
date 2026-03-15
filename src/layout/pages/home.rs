use ratatui::{Frame, layout::{Rect}, widgets::{Block, Borders}};

use crate::{global_state::{ApiData, Roudy}, layout::pages::sub_pages::home::{playlist_tracks::render_playlist_tracks_subpage, playlists::render_playlist_subpage}};


pub fn render_home_page(frame: &mut Frame, chunk: Rect, global_state: &Roudy, api_data: &ApiData) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT);
    let block_area = block.inner(chunk);
    frame.render_widget(block, chunk);

    match global_state.homepage_subpage {
        0 => {
            render_playlist_subpage(frame, block_area, &global_state, &api_data);
        }
        1 => {
            render_playlist_tracks_subpage(frame, block_area, &global_state, &api_data);
        }
        _ => {}
    }

}