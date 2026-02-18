use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};
use crate::{
    global_state::{ApiData, ErrorState, Roudy, RoudyData},
    layout::{components::header::render_header_comp, pages::{
        errors_status::render_errors_status_page, home::render_home_page, profile::render_profile_page
    }}

};

use crate::layout::pages::login::render_login_page;



fn render_main_page(frame: &mut Frame, roudy: &Roudy, api_data: &ApiData, error_state: &ErrorState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(70),
            Constraint::Percentage(20),
        ])
        .split(frame.area());

    render_header_comp(frame, chunks[0], roudy.selected_tab);
    match roudy.selected_tab {
        0 => {
            render_home_page(frame, chunks[1], api_data);
        },
        1 => {
            render_profile_page(frame, chunks[1], api_data);
        },
        2 => {
            render_errors_status_page(frame, chunks[1], error_state);
        }
        _ => {}
    }
}

pub fn ui(frame: &mut Frame, roudy: &Roudy, roudy_data: &RoudyData, api_data: &ApiData, error_state: &ErrorState) {
    if roudy.logged_in {
        render_main_page(frame, &roudy, &api_data, &error_state);
    } else {
        render_login_page(frame, &roudy_data, &error_state);
    }

}
