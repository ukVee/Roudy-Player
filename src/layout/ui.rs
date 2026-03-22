use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};
use crate::{
    global_state::{ApiData, ErrorState, Roudy, RoudyData, SelectedTab},
    layout::{components::{header::render_header_comp, track_bar::render_track_bar}, pages::{
        errors_status::render_errors_status_page, home::render_home_page, profile::render_profile_page, test::render_test_page
    }}

};

use crate::layout::pages::login::render_login_page;



fn render_main_page(frame: &mut Frame, global_state: &Roudy, api_data: &ApiData, error_state: &ErrorState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(70),
            Constraint::Percentage(20),
        ])
        .split(frame.area());

    match global_state.selected_tab {
        SelectedTab::Home => {
            render_header_comp(frame, chunks[0], 0);
            render_home_page(frame, chunks[1], &global_state, &api_data);
        },
        SelectedTab::Profile => {
            render_header_comp(frame, chunks[0], 1);
            render_profile_page(frame, chunks[1], &api_data);
        },
        SelectedTab::ErrorStatus => {
            render_header_comp(frame, chunks[0], 2);
            render_errors_status_page(frame, chunks[1], &error_state);
        }
        SelectedTab::Test => {
            render_header_comp(frame, chunks[0], 3);
            render_test_page(frame, chunks[1], &api_data);
        }
    }
    render_track_bar(frame, chunks[2], &api_data);
}

pub fn ui(frame: &mut Frame, roudy: &Roudy, roudy_data: &RoudyData, api_data: &ApiData, error_state: &ErrorState) {
    if roudy.logged_in {
        render_main_page(frame, &roudy, &api_data, &error_state);
    } else {
        render_login_page(frame, &roudy_data, &error_state);
    }

}
