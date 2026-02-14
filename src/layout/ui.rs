use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Paragraph, Wrap},
};
use crate::{
    global_state::{ApiData, ErrorState, Roudy, RoudyData},
    layout::{components::header::{header, nav_bar}, pages::{
        home::render_home_page, profile::render_profile_page
    }}

};

use crate::layout::pages::login::render_login_page;



fn render_main_page(frame: &mut Frame, roudy: &Roudy, roudy_data: &RoudyData, api_data: &ApiData, _error_state: &ErrorState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(70),
            Constraint::Percentage(20),
        ])
        .split(frame.area());

    let (header, header_area) = header(chunks[0]);
    frame.render_widget(header, chunks[0]);
    frame.render_widget(nav_bar(roudy.selected_tab), header_area);
    match roudy.selected_tab {
        0 => {//home page
            render_home_page(frame, chunks[1]);
        },
        1 => {//profile page
            render_profile_page(frame, chunks[1], api_data);
        },
        _ => {}
    }
}

pub fn ui(frame: &mut Frame, roudy: &Roudy, roudy_data: &RoudyData, api_data: &ApiData, error_state: &ErrorState) {
    if roudy.logged_in {
        render_main_page(frame, &roudy, &roudy_data, &api_data, &error_state);
    } else {
        render_login_page(frame, &roudy_data, &error_state);
    }

}
