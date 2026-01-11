use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::event::KeyCode,
};
use std::io::Stdout;

use crate::api::soundcloud::login_to_sc;
use crate::event::polling::setup_event_polling;
use crate::layout::ui::ui;
use crate::api::server::start_server;
use crate::helpers::parse_query_params::parse_query_params;
use crate::global_state::{
    GlobalState,
    ErrorState,
};
use crate::types::{
    PollEvent,
    ServerEvent,
};

pub async fn event_loop(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> std::io::Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut keybind_receiver = setup_event_polling();
    let mut server_receiver = start_server();
    let mut global_state = GlobalState::new();
    let mut error_state = ErrorState::new();

    loop {
        if let Ok(polling_event) = keybind_receiver.try_recv() {
            match polling_event {
                PollEvent::Input(key) => {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                        keybind_receiver.close();
                        server_receiver.close();
                        break;
                    } else if key.code == KeyCode::Char('l') && !global_state.logged_in {
                        if let Ok(soundcloud_auth) = login_to_sc().await  {
                            global_state.soundcloud_url = Some(soundcloud_auth.auth_url.to_string());
                        }
                    }
                }
                PollEvent::Tick => {

                }
            }
        }

        if let Ok(server_event) = server_receiver.try_recv() {
            match server_event {
                ServerEvent::Url(url) => {
                    match parse_query_params(url) {
                        Some(param) => {
                            global_state.authorization_code = Some(param);
                        }
                        None => {
                            error_state.failed_to_parse_params = true;
                        }
                    }
                }
                ServerEvent::Shutdown => {
                    server_receiver.close();
                }
            }
        }
                
        terminal.draw(|f| {
            ui(f, &global_state, &error_state);
        })?;

    }
    Ok(terminal)
}