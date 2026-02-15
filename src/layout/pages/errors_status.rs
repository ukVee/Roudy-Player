use ratatui::{Frame, crossterm::style::Color, layout::{Constraint, Layout, Rect}, style::Style, widgets::Paragraph};

use crate::global_state::ErrorState;



pub fn render_errors_status_page(frame: &mut Frame, chunk: Rect, error_state: &ErrorState) {
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            
        ])
        .split(chunk);

    let paragraph_one = match error_state.csrf_token_does_not_match {
        true => {
            Paragraph::new("CSRF token does not match: true")
                .style(Style::new().red())
        }
        false => {
            Paragraph::new("CSRF token does not match: false")
                .style(Style::new().green())
        }
    };
    let paragraph_two = match error_state.failed_to_mount_client_request_handler {
        true => {
            Paragraph::new("Failed to mount client request handler: true")
                .style(Style::new().red())
        }
        false => {
            Paragraph::new("Failed to mount client request handler: false")
                .style(Style::new().green())
        }
    };
    let paragraph_three = match error_state.failed_to_parse_code_param {
        true => {
            Paragraph::new("Failed to parse code param from redirect: true")
                .style(Style::new().red())
        }
        false => {
            Paragraph::new("Failed to parse code param from redirect: false")
                .style(Style::new().green())
        }
    };
    let paragraph_four = match error_state.failed_to_parse_csrf_param {
        true => {
            Paragraph::new("Failed to parse CSRF param: true")
                .style(Style::new().red())
        }
        false => {
            Paragraph::new("Failed to parse CSRF param: false")
                .style(Style::new().green())
        }
    };
    let paragraph_five = match error_state.failed_to_shutdown_server {
        true => {
            Paragraph::new("Failed to shutdown http server: true")
                .style(Style::new().red())
        }
        false => {
            Paragraph::new("Failed to shutdown http server: false")
                .style(Style::new().green())
        }
    };
    frame.render_widget(paragraph_one, chunks[0]);
    frame.render_widget(paragraph_two, chunks[1]);
    frame.render_widget(paragraph_three, chunks[2]);
    frame.render_widget(paragraph_four, chunks[3]);
    frame.render_widget(paragraph_five, chunks[4]);
}