use ratatui::{
    widgets::{
        Block,
        Borders,
    },
    layout::{
        Alignment,
    },
};


pub fn header() -> Block<'static> {
    Block::default()
        .title("Roudy Music Player")
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center)
}
