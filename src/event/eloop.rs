use oauth2::TokenResponse;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::event::KeyCode,
};
use std::{
    fs::File,
    io::{Stdout, Write},
};
use crate::{
    api::{server::start_server, soundcloud::login_to_sc},
    event::polling::setup_event_polling,
    global_state::{ErrorState, GlobalState},
    helpers::parse_query_params::parse_query_params,
    layout::ui::ui,
    types::{AuthCredentials, GetAccessToken, PollEvent, ServerEvent},
};

pub async fn event_loop(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut keybind_receiver = setup_event_polling();
    let (mut server_receiver, shutdown_server) = start_server().await?;
    let mut shutdown_server = Some(shutdown_server);
    let mut global_state = GlobalState::new();
    let mut error_state = ErrorState::new();
    let mut get_access_token: GetAccessToken = None;
    let mut csrf_token: Option<oauth2::CsrfToken> = None;
    let auth_credentials_path = "auth_credentials.json";
    
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
                                global_state.login_url = Some(soundcloud_auth.auth_url);
                                get_access_token = Some(soundcloud_auth.get_access_token);
                                csrf_token = Some(soundcloud_auth.csrf_token);
                            }
                            Err(e) => {
                                panic!("Failed to make login url: \n {e}")
                            }

                        }

                        
                    }
                }
                PollEvent::Tick => {}
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
                                    error_state.csrf_token_does_not_match = true;
                                }
                            }
                        }
                        None => {
                            error_state.failed_to_parse_state_param = true;
                        }
                    }
                    match parsed_params.authorization_code {
                        Some(code) => {
                            if let Some(exchange_code) = get_access_token.take() {
                                let auth = (exchange_code)(code).await?;
                                
                                let auth_credentials = AuthCredentials {
                                    access_token: auth.access_token().secret().clone(),
                                    refresh_token: auth
                                        .refresh_token()
                                        .expect("Failed to get access token.")
                                        .secret()
                                        .clone(),
                                    expires_in: format!(
                                        "{:?}",
                                        &auth
                                            .expires_in()
                                            .expect("Failed to get auth expiration date.")
                                    ),
                                };
                                let json_auth_cred = serde_json::to_string_pretty(&auth_credentials)?;
                                let mut file = File::create(auth_credentials_path)?;
                                file.write_all(json_auth_cred.as_bytes())?;
                                if let Some(shutdown) = shutdown_server.take() {
                                    if let Err(_) = shutdown.send(()) {
                                        error_state.failed_to_shutdown_server = true;
                                    }
                                }
                            }
                        }
                        None => {
                            error_state.failed_to_parse_code_param = true;
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
