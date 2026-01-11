use ratatui::{
    Frame, 
    widgets::{
        Block,    
        Paragraph,
        Borders,
    },
    layout::{
        Constraint,
        Direction,
        Layout,
    },

};

use crate::layout::components::header::header;
use crate::global_state::{
    ErrorState,    
    GlobalState,
};



pub fn ui(frame: &mut Frame, global_state: &GlobalState, error_state: &ErrorState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(70),
            Constraint::Percentage(20),
        ])
        .split(frame.area());
    
    frame.render_widget(header(), chunks[0]);

    let text = match &global_state.soundcloud_url {
        Some(url) => format!("Login URL: {}", url),
        None => "Press L to login to SoundCloud".to_string(),
    };


    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, frame.area());
}