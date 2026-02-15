use ratatui::{
    Frame, layout::{Alignment, Rect}, style::Style, symbols, widgets::{Block, Borders, Tabs}
};


pub fn render_header_comp(frame: &mut Frame, chunk: Rect, tab_num: usize) {
    let header = Block::default()
        .title("Roudy SoundCloud Player")
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center);
    let header_area = header.inner(chunk);
        let tab = Tabs::new(vec!["Home", "Profile", "Errors Status"])
        .style(Style::default().white())
        .highlight_style(Style::default().yellow())
        .divider(symbols::DOT)
        .select(tab_num);
    
    frame.render_widget(header, chunk);
    frame.render_widget(tab, header_area);
}