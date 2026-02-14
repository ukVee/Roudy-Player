use ratatui::{Frame, layout::Rect, widgets::{Block, Borders}};




pub fn render_home_page(frame: &mut Frame, chunk: Rect) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT);
    frame.render_widget(block, chunk);
}