use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Paragraph, Wrap},
};
use crate::{
    global_state::{ErrorState, Roudy, RoudyData},
    layout::pages::{
        profile::render_profile_page,
        home::render_home_page,
    },
    layout::components::{
        header::{header, nav_bar}
}

};

use crate::layout::pages::login::render_login_page;



fn render_main_page(frame: &mut Frame, roudy: &Roudy, roudy_data: &RoudyData, _error_state: &ErrorState) {
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
            frame.render_widget(render_home_page(), chunks[1]);
        },
        1 => {//profile page
            frame.render_widget(render_profile_page(), chunks[1]);
        },
        _ => {}
    }
}

pub fn ui(frame: &mut Frame, roudy: &Roudy, roudy_data: &RoudyData, error_state: &ErrorState) {
    if roudy.logged_in {
        render_main_page(frame, &roudy, &roudy_data, &error_state);
    } else {
        render_login_page(frame, &roudy_data, &error_state);
    }

}
