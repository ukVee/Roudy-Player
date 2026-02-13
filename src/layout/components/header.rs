use ratatui::{
    style::{Style},
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Tabs},
    symbols
};


pub fn header(area: Rect) -> (Block<'static>, Rect) {
    let header = Block::default()
        .title("Roudy SoundCloud Player")
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center);
    let header_area = header.inner(area);
    (header, header_area)
        
}

pub fn nav_bar(tab_num: usize) -> Tabs<'static> {
    Tabs::new(vec!["Home", "Profile"])
        .style(Style::default().white())
        .highlight_style(Style::default().yellow())
        .divider(symbols::DOT)
        .select(tab_num)
}