use tokio::sync::mpsc::{Receiver, Sender};

use crate::{credentials_manager::{ CredentialsEvent}, global_state::{ErrorMessage, ErrorState}, helpers::parse_query_params::parse_query_params, types::{GetAccessToken, ServerEvent}};



pub async fn auth_server_listener(
    server_receiver: &mut Receiver<ServerEvent>,
    csrf_token: &Option<oauth2::CsrfToken>,
    error_state: &mut  ErrorState,
    credentials_messenger: & Sender<CredentialsEvent>,
    shutdown_auth_server: & Sender<()>,
    get_access_token: &mut GetAccessToken,
) {
            if let Ok(server_event) = server_receiver.try_recv() {
            match server_event {
                ServerEvent::Url(url) => {
                    let parsed_params = parse_query_params(url);
                    match parsed_params.csrf_state {
                        Some(state) => {
                            if let Some(token) = csrf_token {
                                if &state != token.secret() {
                                    ErrorState::update(
                                        error_state,
                                        ErrorMessage::CSRFTokenDoesntMatch,
                                    );
                                }
                            }
                        }
                        None => {
                            ErrorState::update(
                                error_state,
                                ErrorMessage::FailedCSRFParamParse,
                            );
                        }
                    }
                    match parsed_params.authorization_code {
                        Some(code) => {
                            if let Some(exchange_code) = get_access_token.take() {
                                let auth_token = (exchange_code)(code).await.expect("Should get auth token");
                                let _ = credentials_messenger.send(CredentialsEvent::SaveToken(auth_token)).await;
                                
                                    if let Err(_) = shutdown_auth_server.send(()).await {
                                        ErrorState::update(
                                            error_state,
                                            ErrorMessage::FailedServerShutdown,
                                        );
                                    }
                                
                            }
                        }
                        None => {
                            ErrorState::update(
                                error_state,
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
}