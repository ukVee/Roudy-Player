use crate::{
    api::{
        request_handler::{ApiOutput, ClientEvent, mount_client_request_handler}, server::start_server, soundcloud::auth_client::login_to_sc
    },
    event::keypress_polling::setup_event_polling,
    global_state::{ApiData, ApiDataMessage, ErrorMessage, ErrorState, Roudy, RoudyData, RoudyDataMessage, RoudyMessage},
    helpers::{parse_query_params::parse_query_params, refresh_token::save_token_to_file},
    layout::ui::ui,
    types::{GetAccessToken, PollEvent, ServerEvent},
};
use ratatui::{Terminal, backend::CrosstermBackend, crossterm::event::KeyCode};
use std::io::Stdout;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

pub async fn event_loop(
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
) -> anyhow::Result<(
    Terminal<CrosstermBackend<Stdout>>,
    Option<tokio::sync::oneshot::Sender<()>>,
)> {
    let mut keybind_receiver = setup_event_polling();
    let (mut server_receiver, shutdown_server) = start_server().await?;
    let mut shutdown_server = Some(shutdown_server);
    let mut data_receiver: Option<Receiver<ApiOutput>> = None;
    let mut req_api_data: Option<Sender<ClientEvent>> = None;

    let mut global_state = Roudy::new();
    let mut roudy_data = RoudyData::new();
    let mut error_state = ErrorState::new();
    let mut api_data = ApiData::new();

    let mut get_access_token: GetAccessToken = None;
    let mut csrf_token: Option<oauth2::CsrfToken> = None;
    const PAGES: usize = 3;
    loop {
        if let Ok(polling_event) = keybind_receiver.try_recv() {
            match polling_event {
                PollEvent::Input(key) => {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                        keybind_receiver.close();
                        server_receiver.close();
                        if let Some(rx) = data_receiver.as_mut() {
                            rx.close();
                        }
                        if let Some(tx) = req_api_data {
                            let _ = tx.send(ClientEvent::Shutdown).await;
                        }
                        break;
                    } else if key.code == KeyCode::Char('l') && !global_state.logged_in {
                        match login_to_sc().await {
                            Ok(soundcloud_auth) => {
                                RoudyData::update(
                                    &mut roudy_data,
                                    RoudyDataMessage::SetLoginURL(soundcloud_auth.auth_url),
                                );
                                get_access_token = Some(soundcloud_auth.get_access_token);
                                csrf_token = Some(soundcloud_auth.csrf_token);
                            }
                            Err(e) => {
                                panic!("Failed to make login url: \n {e}")
                            }
                        }
                    } else if key.code == KeyCode::Tab && global_state.logged_in {
                        let mut new_tab = global_state.selected_tab + 1;
                        if new_tab >= PAGES {
                            new_tab = 0;
                        }
                        Roudy::update(&mut global_state, RoudyMessage::ChangeTab(new_tab));
                        match new_tab {
                            0 => {
                                if let Some(sender) = req_api_data.as_ref() {
                                    let _ = sender.send(ClientEvent::GetPlaylists).await;
                                }
                            }
                            1 => {
                                if let Some(sender) = req_api_data.as_ref() {
                                    let _ = sender.send(ClientEvent::GetProfile).await;
                                }
                            }
                            _ => {}
                        }
                    }
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
                                    ErrorState::update(
                                        &mut error_state,
                                        ErrorMessage::CSRFTokenDoesntMatch,
                                    );
                                }
                            }
                        }
                        None => {
                            ErrorState::update(
                                &mut error_state,
                                ErrorMessage::FailedCSRFParamParse,
                            );
                        }
                    }
                    match parsed_params.authorization_code {
                        Some(code) => {
                            if let Some(exchange_code) = get_access_token.take() {
                                let auth_token = (exchange_code)(code).await?;
                                if let Some(saved_token_path) = save_token_to_file(auth_token) {
                                    Roudy::update(&mut global_state, RoudyMessage::Login);
                                    RoudyData::update(
                                        &mut roudy_data,
                                        RoudyDataMessage::SetTokenPath(saved_token_path),
                                    );
                                    match mount_client_request_handler(&roudy_data).await {
                                        Ok(sender) => {
                                            // let _ = sender.send(ClientEvent::GetProfile).await;
                                            req_api_data = Some(sender.0);
                                            data_receiver = Some(sender.1);
                                        }
                                        Err(_) => {
                                            ErrorState::update(
                                                &mut error_state,
                                                ErrorMessage::FailedMountClientRequestHandler,
                                            );
                                        }
                                    }
                                }
                                if let Some(shutdown) = shutdown_server.take() {
                                    if let Err(_) = shutdown.send(()) {
                                        ErrorState::update(
                                            &mut error_state,
                                            ErrorMessage::FailedServerShutdown,
                                        );
                                    }
                                }
                            }
                        }
                        None => {
                            ErrorState::update(
                                &mut error_state,
                                ErrorMessage::FailedCodeParamParse,
                            );
                        }
                    }
                }
                ServerEvent::Shutdown => {
                    server_receiver.close();
                }
            }
        }

        if let Some(rx) = data_receiver.as_mut() {
            if let Ok(api_event) = rx.try_recv() {
                match api_event {
                    ApiOutput::Error(message) => {
                        ErrorState::update( &mut error_state, ErrorMessage::ApiError(message));
                    }
                    ApiOutput::Profile(data) => {
                        ApiData::update(&mut api_data, ApiDataMessage::ProfileFetched(data));
                    }
                    ApiOutput::Playlists(data) => {
                        ApiData::update(&mut api_data, ApiDataMessage::PlaylistsFetched(data));
                    }
                }
            }
        }

        terminal.draw(|f| {
            ui(f, &global_state, &roudy_data, &api_data, &error_state);
        })?;
    }
    Ok((terminal, shutdown_server))
}
