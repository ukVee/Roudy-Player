use ratatui::{Frame, layout::Rect, widgets::{Block, Borders, Paragraph, Wrap}};

use crate::{ global_state::ApiData};




pub fn render_profile_page(frame: &mut Frame, chunk: Rect, api_data: &ApiData) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT);
    let block_area = block.inner(chunk);
    frame.render_widget(block, chunk);
    if let Some(data) = &api_data.profile {
        let paragraph = Paragraph::new(data.to_string())
            .wrap(Wrap { trim: true});
        frame.render_widget(paragraph, block_area);
    }
}