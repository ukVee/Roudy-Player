use ratatui::widgets::{Block, Borders};




pub fn render_home_page() -> Block<'static> {
    Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
}