use ratatui::{Frame, layout::{Alignment, Constraint, Direction, Layout, Rect}, widgets::{Block, Borders, Paragraph}};

use crate::global_state::{ApiData, Roudy};



pub fn render_playlist_subpage(frame: &mut Frame, block_area: Rect, global_state: &Roudy, api_data: &ApiData) {

    const PLAYLIST_ROW_HEIGHT: u16 = 3;
    let available_rows = block_area.height;
    let viewable_playlists = available_rows / (PLAYLIST_ROW_HEIGHT + 1);

    if let Some(data) = &api_data.playlists {
        let mut constraints: Vec<Constraint> = Vec::new();
        for _ in 0..viewable_playlists {
            constraints.push(Constraint::Min(PLAYLIST_ROW_HEIGHT));
        }

        let playlists_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(constraints)
            .split(block_area);
        let offset = global_state.homepage_playlist_scroll_offset as usize;
        let mut count = 0;

        for playlist in data.iter().skip(offset).take(viewable_playlists as usize) {

            let playlist_block = Block::default()
                .borders(Borders::all())
                .title(playlist.permalink.clone())
                .title_alignment(Alignment::Center);
            let inner_playlist_block = playlist_block.inner(playlists_layout[count]);
            frame.render_widget(playlist_block, playlists_layout[count]);

            let inner_playlist_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(inner_playlist_block);

            let duration_p = Paragraph::new(format!("Playlist Time: {}",playlist.duration.to_string()));
            let total_tracks_p = Paragraph::new(format!("Total Tracks: {}",playlist.track_count.to_string()));
            frame.render_widget(duration_p, inner_playlist_layout[0]);
            frame.render_widget(total_tracks_p, inner_playlist_layout[1]);
            count += 1;
        }
    }
}