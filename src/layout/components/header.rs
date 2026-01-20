use ratatui::{
    layout::Alignment,
    widgets::{Block, Borders},
};


pub fn header() -> Block<'static> {
    Block::default()
        .title("Roudy SoundCloud Player")
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center)
}
