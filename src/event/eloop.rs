use crate::{
    api::request_handler::{ApiOutput, ApiRequestHandler, ClientEvent},
    audio::audio_handler::AudioHandler,
    auth::{
        credentials_manager::{CredentialsEvent, CredentialsManager},
        server::start_server,
    },
    event::{
        api_output_listener::api_listener,
        auth_server_listener::auth_server_listener,
        credentials_output_listener::{credentials_listener},
        keybind::{
            keypress_output_listener::keypress_listener, keypress_polling::setup_event_polling,
        },
    },
    global_state::{ApiData, ErrorMessage, ErrorState, Roudy, RoudyData, RoudyMessage},
    layout::ui::ui,
    types::GetAccessToken,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::Stdout;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

pub async fn event_loop(
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
) -> anyhow::Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut keybind_receiver = setup_event_polling();
    let (mut server_receiver, shutdown_auth_server) = start_server().await?;
    let mut audio_handler = AudioHandler::mount();
    let mut audio_receiver = audio_handler.audio_messeneger.clone();

    let mut api_data_receiver: Option<Receiver<ApiOutput>> = None;
    let mut req_api_data: Option<Sender<ClientEvent>> = None;

    let mut access_token: Option<String> = None;

    let mut global_state = Roudy::new();
    let mut roudy_data = RoudyData::new();
    let mut error_state = ErrorState::new();
    let mut api_data = ApiData::new();
    let (credentials_messenger, mut credentials_receiver) =
        CredentialsManager::mount().await.cred_channels;
    let mut request_handler_mounted = false;

    let mut get_access_token: GetAccessToken = None;
    let mut csrf_token: Option<oauth2::CsrfToken> = None;

    loop {
        tokio::select! {
            Some(msg) = credentials_receiver.recv() => {
                let new_token = credentials_listener(msg, &mut roudy_data, &mut get_access_token, &mut csrf_token, &mut access_token, &mut error_state).await;
                if new_token && req_api_data.is_some() && access_token.is_some() {
                    let updated_token = access_token.as_ref().expect("should have");
                    let _ = req_api_data.as_ref().expect("should have").send(ClientEvent::UpdateAccessToken(updated_token.clone())).await;
                }
            }
            Some(msg) = keybind_receiver.recv() => {
                let shutdown = keypress_listener(msg,&req_api_data,&mut global_state,&mut api_data,).await;
                if shutdown {
                    keybind_receiver.close();
                    server_receiver.close();
                    if let Some(rx) = api_data_receiver.as_mut() {
                        rx.close();
                    }
                    if let Some(tx) = req_api_data {
                        let _ = tx.send(ClientEvent::Shutdown).await;
                    }
                    credentials_receiver.close();
                    let _ = credentials_messenger.send(CredentialsEvent::Shutdown).await;
                    let _ = shutdown_auth_server.send(()).await;
                    break;
                }
            }
            Some(msg) = server_receiver.recv(), if !server_receiver.is_closed() => {
                let shutdown_auth_server = auth_server_listener(msg,&csrf_token,&mut error_state,&credentials_messenger,&shutdown_auth_server,&mut get_access_token,).await;
                if shutdown_auth_server {
                    server_receiver.close();
                }
            }
            Some(msg) = async {
                match api_data_receiver.as_mut() {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                api_listener(msg,&mut audio_receiver,&mut audio_handler,&mut global_state,&mut api_data,&mut error_state);
            }
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

        terminal.draw(|f| {
            ui(f, &global_state, &roudy_data, &api_data, &error_state);
        })?;
    }
    Ok(terminal)
}
