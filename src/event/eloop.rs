use crate::{
    api::{
        request_handler::{ApiOutput, ApiRequestHandler, ClientEvent},
        server::start_server,
    },
    credentials_manager::CredentialsManager,
    event::{api_output_listener::api_listener, auth_server_listener::auth_server_listener, credentials_output_listener::{CredentialsListenerMessage, credentials_listener}, keybind::{keypress_output_listener::{keypress_listener, KeypressListenerStatus}, keypress_polling::setup_event_polling}},
    global_state::{
        ApiData, ErrorMessage, ErrorState, Roudy, RoudyData, 
        RoudyMessage,
    },
    layout::ui::ui,
    types::GetAccessToken,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::Stdout;
use tokio::sync::{mpsc::Receiver,};
use tokio::sync::mpsc::Sender;


pub async fn event_loop(
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
) -> anyhow::Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut keybind_receiver = setup_event_polling();
    let (mut server_receiver, shutdown_auth_server) = start_server().await?;

    let mut api_data_receiver: Option<Receiver<ApiOutput>> = None;
    let mut req_api_data: Option<Sender<ClientEvent>> = None;

    let mut access_token: Option<String> = None;

    let mut global_state = Roudy::new();
    let mut roudy_data = RoudyData::new();
    let mut error_state = ErrorState::new();
    let mut api_data = ApiData::new();
    let (credentials_messenger, mut credentials_receiver) =CredentialsManager::mount().await.cred_channels;
    let mut request_handler_mounted = false;

    let mut get_access_token: GetAccessToken = None;
    let mut csrf_token: Option<oauth2::CsrfToken> = None;
    
    loop {
        let message = credentials_listener(&mut credentials_receiver, &mut roudy_data, &mut get_access_token, &mut csrf_token, &mut access_token, &mut error_state).await;

        if message == CredentialsListenerMessage::NewTokenReceived && req_api_data.is_some() && access_token.is_some() {
            let updated_token = access_token.as_ref().expect("should have");
            let _ = req_api_data.as_ref().expect("should have").send(ClientEvent::UpdateAccessToken(updated_token.clone())).await;
        }

        let signal = keypress_listener(&mut keybind_receiver, &mut server_receiver, &mut api_data_receiver, &req_api_data, &mut credentials_receiver, &credentials_messenger, &shutdown_auth_server, &mut global_state).await;

        if signal == KeypressListenerStatus::Shutdown {
            break;
        }
        if !server_receiver.is_closed() {
            auth_server_listener(&mut server_receiver, &csrf_token, &mut error_state, &credentials_messenger, &shutdown_auth_server, &mut get_access_token).await;

        }

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

        api_listener(&mut api_data_receiver, &mut global_state, &mut api_data, &mut error_state);

        terminal.draw(|f| {
            ui(f, &global_state, &roudy_data, &api_data, &error_state);
        })?;
    }
    Ok(terminal)
}
