use crate::{
    global_state::{ErrorState, RoudyData}
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Paragraph, Wrap},
};

pub fn render_login_page(frame: &mut Frame, roudy_data: &RoudyData, _error_state: &ErrorState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(100)
        ])
        .split(frame.area());

    let text = match &roudy_data.login_url {
        Some(url) => url.to_string(),
        None => "There has been a bug, please restart. \n If that doesnt work then delete your auth_credentials.json".to_string()
    };
    
    
    let paragraph = Paragraph::new(text)
        .wrap(Wrap {trim: true});
    frame.render_widget(paragraph, chunks[0]);
}