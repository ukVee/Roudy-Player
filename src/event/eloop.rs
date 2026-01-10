use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::event::KeyCode,
};
use std::io::Stdout;

use crate::event::polling::setup_event_polling;
use crate::types::PollEvent;

pub async fn event_loop(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> std::io::Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut rx = setup_event_polling();
    loop {
        terminal.draw(|_f| {
            // UI code here
        })?;

        if let Some(event) = rx.recv().await {
            match event {
                PollEvent::Input(key) => {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                        rx.close();
                        break;
                    }
                }
                PollEvent::Tick => {}
            }
        }
    }
    Ok(terminal)
}