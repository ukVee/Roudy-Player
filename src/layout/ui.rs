use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Paragraph, Wrap},
};
use crate::{
    global_state::{ErrorState, GlobalState},
    layout::components::header::header,
};

fn render_login_page(frame: &mut Frame, global_state: &GlobalState, _error_state: &ErrorState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(100)
        ])
        .split(frame.area());

    let text = match &global_state.login_url {
        Some(login_url) => format!("Login URL: {}", login_url),
        None => "Press L to login to SoundCloud".to_string(),
    };
    let paragraph = Paragraph::new(text)
        .wrap(Wrap {trim: true});
    frame.render_widget(paragraph, chunks[0]);
}

fn render_main_page(frame: &mut Frame, global_state: &GlobalState, _error_state: &ErrorState) {
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
}

pub fn ui(frame: &mut Frame, global_state: &GlobalState, error_state: &ErrorState) {
    
    if global_state.logged_in {
        render_main_page(frame, global_state, error_state);
    } else {
        render_login_page(frame, global_state, error_state);
    }



}
