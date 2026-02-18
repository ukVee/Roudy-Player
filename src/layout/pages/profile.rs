use ratatui::{Frame, layout::{Constraint, Layout, Rect}, widgets::{Block, Borders, Paragraph, Wrap}};

use crate::{ global_state::ApiData};




pub fn render_profile_page(frame: &mut Frame, chunk: Rect, api_data: &ApiData) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT);
    let block_area = block.inner(chunk);
    let main_layout = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(block_area);
    let right_side_layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(40),
        ])
        .split(main_layout[1]);

    frame.render_widget(block, chunk);
    if let Some(api_profile) = &api_data.profile {
        let username_p = Paragraph::new(api_profile.username.clone())
            .wrap(Wrap { trim: true});
        let description_p = Paragraph::new(api_profile.description.clone())
            .wrap(Wrap { trim: true});
        let plan_p = Paragraph::new(api_profile.plan.clone())
            .wrap(Wrap { trim: true});
        frame.render_widget(username_p, right_side_layout[0]);
        frame.render_widget(plan_p, right_side_layout[1]);
        frame.render_widget(description_p, right_side_layout[2]);
    } else {
        //add loading bar or sum
    }
}