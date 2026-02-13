use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::event::KeyCode,
};
use std::{
    io::{Stdout},
};
use tokio::sync::oneshot::Sender;
use crate::{
    api::{server::start_server, soundcloud::auth_client::login_to_sc},
    event::keypress_polling::setup_event_polling,
    global_state::{ErrorState, ErrorMessage, Roudy, RoudyMessage, RoudyData, RoudyDataMessage},
    helpers::{parse_query_params::parse_query_params, refresh_token::save_token_to_file},
    layout::ui::ui,
    types::{GetAccessToken, PollEvent, ServerEvent},
};

pub async fn event_loop(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<(Terminal<CrosstermBackend<Stdout>>, Option<Sender<()>>)> {
    let mut keybind_receiver = setup_event_polling();
    let (mut server_receiver, shutdown_server) = start_server().await?;
    let mut shutdown_server = Some(shutdown_server);

    let mut global_state = Roudy::new();
    let mut roudy_data = RoudyData::new();
    let mut error_state = ErrorState::new();

    let mut get_access_token: GetAccessToken = None;
    let mut csrf_token: Option<oauth2::CsrfToken> = None;
    const PAGES: usize = 2;
    loop {
        if let Ok(polling_event) = keybind_receiver.try_recv() {
            match polling_event {
                PollEvent::Input(key) => {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                        keybind_receiver.close();
                        server_receiver.close();
                        break;
                    } else if key.code == KeyCode::Char('l') && !global_state.logged_in {
                        match login_to_sc().await {
                            Ok(soundcloud_auth) =>  {
                                RoudyData::update(&mut roudy_data, RoudyDataMessage::SetLoginURL(soundcloud_auth.auth_url));
                                get_access_token = Some(soundcloud_auth.get_access_token);
                                csrf_token = Some(soundcloud_auth.csrf_token);
                            }
                            Err(e) => {
                                panic!("Failed to make login url: \n {e}")
                            }

                        } 
                    } else if key.code == KeyCode::Tab && global_state.logged_in {
                        let mut new_tab = global_state.selected_tab +1;
                        if new_tab >= PAGES {
                            new_tab = 0;
                        }
                        Roudy::update(&mut global_state, RoudyMessage::ChangeTab(new_tab));
                    }
                }
                PollEvent::Tick => {
                    
                }
            }
        }

        if let Ok(server_event) = server_receiver.try_recv() {
            match server_event {
                ServerEvent::Url(url) => {
                    let parsed_params = parse_query_params(url);
                    match parsed_params.csrf_state {
                        Some(state) => {
                            if let Some(token) = &csrf_token {
                                if &state != token.secret() {
                                    ErrorState::update(&mut error_state, ErrorMessage::CSRFTokenDoesntMatch);
                                }
                            }
                        }
                        None => {
                            ErrorState::update(&mut error_state, ErrorMessage::FailedCSRFParamParse);
                        }
                    }
                    match parsed_params.authorization_code {
                        Some(code) => {
                            if let Some(exchange_code) = get_access_token.take() {
                                let auth_token = (exchange_code)(code).await?;
                                if let Some(saved_token_path) = save_token_to_file(auth_token) {
                                    Roudy::update(&mut global_state, RoudyMessage::Login);
                                    RoudyData::update(&mut roudy_data, RoudyDataMessage::SetTokenPath(saved_token_path));
                                }
                                if let Some(shutdown) = shutdown_server.take() {
                                    if let Err(_) = shutdown.send(()) {
                                        ErrorState::update(&mut error_state, ErrorMessage::FailedServerShutdown);
                                    }
                                }
                            }
                        }
                        None => {
                            ErrorState::update(&mut error_state, ErrorMessage::FailedCodeParamParse);
                        }
                    }
                }
                ServerEvent::Shutdown => {
                    server_receiver.close();
                }
            }
        }

        terminal.draw(|f| {
            ui(f, &global_state, &roudy_data, &error_state);
        })?;
    }
    Ok((terminal, shutdown_server))
}
