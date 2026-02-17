use crate::{
    api::{
        request_handler::{ApiOutput, ApiRequestHandler, ClientEvent},
        server::start_server,
        soundcloud::auth_client::login_to_sc,
    },
    credentials_manager::{CredentialsEvent, CredentialsManager, CredentialsOutputEvent},
    event::{api_output_listener::mount_api_listener, keypress_polling::setup_event_polling},
    global_state::{
        ApiData, ApiDataMessage, ErrorMessage, ErrorState, Roudy, RoudyData, RoudyDataMessage,
        RoudyMessage,
    },
    helpers::parse_query_params::parse_query_params,
    layout::ui::ui,
    types::{GetAccessToken, PollEvent, ServerEvent},
};
use ratatui::{Terminal, backend::CrosstermBackend, crossterm::event::KeyCode};
use std::io::Stdout;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

#[derive(PartialEq)]
enum AuthenticationStatus {
    Authenticated,
    Pending,
    NotAuthenticated,
    ReceivedRedirect,
    Started,
    ErrorWhileAuthenticating(String),
}

pub async fn event_loop(
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
) -> anyhow::Result<(
    Terminal<CrosstermBackend<Stdout>>,
    Option<tokio::sync::oneshot::Sender<()>>,
)> {
    let mut keybind_receiver = setup_event_polling();
    let (mut server_receiver, shutdown_server) = start_server().await?;
    let mut shutdown_server = Some(shutdown_server);
    let mut api_data_receiver: Option<Receiver<ApiOutput>> = None;
    let mut req_api_data: Option<Sender<ClientEvent>> = None;

    let mut authentication_status:AuthenticationStatus = AuthenticationStatus::NotAuthenticated;
    let mut access_token: Option<String> = None;

    let mut global_state = Roudy::new();
    let mut roudy_data = RoudyData::new();
    let mut error_state = ErrorState::new();
    let mut api_data = ApiData::new();
    let (credentials_messenger, mut credentials_output) =CredentialsManager::mount().await.cred_channels;
    let mut request_handler_mounted = false;

    let mut get_access_token: GetAccessToken = None;
    let mut csrf_token: Option<oauth2::CsrfToken> = None;
    const PAGES: usize = 3;
    loop {
        if let Ok(cred_message) = credentials_output.try_recv() {
            match cred_message {
                CredentialsOutputEvent::PromptLogin => {
                    authentication_status = AuthenticationStatus::Started;
                    match login_to_sc().await {
                        Ok(soundcloud_auth) => {
                            RoudyData::update(
                                &mut roudy_data,
                                RoudyDataMessage::SetLoginURL(soundcloud_auth.auth_url),
                            );
                            authentication_status = AuthenticationStatus::Pending;
                            get_access_token = Some(soundcloud_auth.get_access_token);
                            csrf_token = Some(soundcloud_auth.csrf_token);
                        }
                        Err(e) => {
                            panic!("Failed to make login url: \n {e}")
                        }
                    };
                }
                CredentialsOutputEvent::AccessToken(token) => {
                    access_token = Some(token);
                }
                CredentialsOutputEvent::NoAccessToken => {

                }
                CredentialsOutputEvent::Error(message) => {

                }
            }
        }
        if let Ok(polling_event) = keybind_receiver.try_recv() {
            match polling_event {
                PollEvent::Input(key) => {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                        keybind_receiver.close();
                        server_receiver.close();
                        if let Some(rx) = api_data_receiver.as_mut() {
                            rx.close();
                        }
                        if let Some(tx) = req_api_data {
                            let _ = tx.send(ClientEvent::Shutdown).await;
                        }
                        credentials_output.close();
                        let _ = credentials_messenger.send(CredentialsEvent::Shutdown).await;
                        break;
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
                                let _ = credentials_messenger.send(CredentialsEvent::SaveToken(auth_token)).await;
                                
                                authentication_status = AuthenticationStatus::ReceivedRedirect;
                                
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
        //figure out a better way
        if !request_handler_mounted {
            match access_token.as_ref() {
                Some(token) => {
                    let channels = ApiRequestHandler::mount(token.clone()).await;
                    req_api_data = Some(channels.api_req_handler_messenger);
                    api_data_receiver = Some(channels.api_data_receiver);
                    request_handler_mounted = true;
                    Roudy::update(&mut global_state, RoudyMessage::Login);
                }
                None => {
                    ErrorState::update(
                        &mut error_state,
                        ErrorMessage::FailedMountApiRequestHandler,
                    );
                }
            }
        }

        mount_api_listener(&mut api_data_receiver, &mut api_data, &mut error_state);

        terminal.draw(|f| {
            ui(f, &global_state, &roudy_data, &api_data, &error_state);
        })?;
    }
    Ok((terminal, shutdown_server))
}
