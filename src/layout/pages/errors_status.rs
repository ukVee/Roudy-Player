use ratatui::{
    Frame, 
    layout::{
        Constraint, Direction::{Horizontal, Vertical}, Layout, Rect
    }, 
    style::Style, widgets::{Paragraph, Wrap}};

use crate::global_state::ErrorState;



pub fn render_errors_status_page(frame: &mut Frame, chunk: Rect, error_state: &ErrorState) {
    let horizontal_layout = Layout::default()
        .direction(Horizontal)
        .margin(1)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50)
        ])
        .split(chunk);
    let chunks = Layout::default()
        .direction(Vertical)
        .margin(1)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            
        ])
        .split(horizontal_layout[0]);

    let paragraph_one = match error_state.csrf_token_does_not_match {
        true => {
            Paragraph::new("CSRF token does not match: true")
                .style(Style::new().red())
                .wrap(Wrap {trim:true})
        }
        false => {
            Paragraph::new("CSRF token does not match: false")
                .style(Style::new().green())
                .wrap(Wrap {trim:true})
        }
    };
    let paragraph_two = match error_state.failed_to_mount_client_request_handler {
        true => {
            Paragraph::new("Failed to mount client request handler: true")
                .style(Style::new().red())
                .wrap(Wrap {trim:true})
        }
        false => {
            Paragraph::new("Failed to mount client request handler: false")
                .style(Style::new().green())
                .wrap(Wrap {trim:true})
        }
    };
    let paragraph_three = match error_state.failed_to_parse_code_param {
        true => {
            Paragraph::new("Failed to parse code param from redirect: true")
                .style(Style::new().red())
                .wrap(Wrap {trim:true})
        }
        false => {
            Paragraph::new("Failed to parse code param from redirect: false")
                .style(Style::new().green())
                .wrap(Wrap {trim:true})
        }
    };
    let paragraph_four = match error_state.failed_to_parse_csrf_param {
        true => {
            Paragraph::new("Failed to parse CSRF param: true")
                .style(Style::new().red())
                .wrap(Wrap {trim:true})
        }
        false => {
            Paragraph::new("Failed to parse CSRF param: false")
                .style(Style::new().green())
                .wrap(Wrap {trim:true})
        }
    };
    let paragraph_five = match error_state.failed_to_shutdown_server {
        true => {
            Paragraph::new("Failed to shutdown http server: true")
                .style(Style::new().red())
                .wrap(Wrap {trim:true})
        }
        false => {
            Paragraph::new("Failed to shutdown http server: false")
                .style(Style::new().green())
                .wrap(Wrap {trim:true})
        }
    };
    let mut  api_error_string = "".to_string();
    for message in &error_state.api_error_log {
        api_error_string.push_str(format!("{} \n", message).as_str());
    }
    let api_error_paragraph = Paragraph::new(api_error_string)
        .wrap(Wrap {trim:true});
    frame.render_widget(api_error_paragraph, horizontal_layout[1]);
    frame.render_widget(paragraph_one, chunks[0]);
    frame.render_widget(paragraph_two, chunks[1]);
    frame.render_widget(paragraph_three, chunks[2]);
    frame.render_widget(paragraph_four, chunks[3]);
    frame.render_widget(paragraph_five, chunks[4]);
}