use ratatui::{Frame, layout::{Alignment, Constraint, Direction, Layout, Rect}, widgets::{Block, Borders, Paragraph}};

use crate::global_state::{ ApiData, Roudy};



pub fn render_playlist_tracks_subpage(frame: &mut Frame, block_area: Rect, global_state: &Roudy, api_data: &ApiData) {
    const PLAYLIST_ROW_HEIGHT: u16 = 3;
    let available_rows = block_area.height;
    let viewable_tracks = available_rows / (PLAYLIST_ROW_HEIGHT + 1);
    
    if let Some(data) = &api_data.playlist_tracks {
        let mut constraints: Vec<Constraint> = Vec::new();

        for _ in 0..viewable_tracks {
            constraints.push(Constraint::Min(PLAYLIST_ROW_HEIGHT));
        }

        let tracks_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(constraints)
            .split(block_area);
        let offset = global_state.homepage_tracks_scroll_offset as usize;

        let mut count = 0;

        for track in data.iter().skip(offset).take(viewable_tracks as usize) {
            let playlist_block = Block::default()
                .borders(Borders::all())
                .title(track.title.clone())
                .title_alignment(Alignment::Center);
            let inner_playlist_block = playlist_block.inner(tracks_layout[count]);
            frame.render_widget(playlist_block, tracks_layout[count]);

            let inner_playlist_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(inner_playlist_block);
            let streamable_p = Paragraph::new(format!("Streamable: {}", track.streamable.to_string()));
            frame.render_widget(streamable_p, inner_playlist_layout[0]);
            if let Some(track_description) = track.description.as_ref() {
                let description_p = Paragraph::new(track_description.to_string());
                frame.render_widget(description_p, inner_playlist_layout[1]);
            }
            count += 1;
        }
    }
}