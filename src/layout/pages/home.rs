use ratatui::{Frame, layout::{self, Alignment, Constraint, Layout, Rect}, widgets::{Block, Borders, Paragraph, Wrap}};

use crate::global_state::ApiData;




pub fn render_home_page(frame: &mut Frame, chunk: Rect, api_data: &ApiData) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT);
    let block_area = block.inner(chunk);
    frame.render_widget(block, chunk);
    
    if let Some(data) = &api_data.playlists {
        let mut constraints: Vec<Constraint> = Vec::new();
        for _ in 0..data.len() {
            constraints.push(Constraint::Min(30));
        }
        let layout = Layout::default()
            .direction(layout::Direction::Vertical)
            .margin(1)
            .constraints(constraints)
            .split(block_area);
        let mut count = 0;
        for playlist in data {
            
            let block = Block::default()
                .borders(Borders::all())
                .title(playlist.permalink.clone())
                .title_alignment(Alignment::Center);
            let inner_playlist_block = block.inner(layout[count]);
            frame.render_widget(block, layout[count]);

            let inner_playlist_layout = Layout::default()
                .direction(layout::Direction::Horizontal)
                .margin(1)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50)
                ])
                .split(inner_playlist_block);

            let duration_p = Paragraph::new(playlist.duration.to_string());
            let total_tracks_p = Paragraph::new(playlist.track_count.to_string());
            frame.render_widget(duration_p, inner_playlist_layout[0]);
            frame.render_widget(total_tracks_p, inner_playlist_layout[1]);
            count += 1;
        }
    }

}