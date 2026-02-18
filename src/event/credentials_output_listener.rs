use tokio::sync::mpsc::Receiver;
use crate::global_state::{ErrorMessage, ErrorState};
use crate::types::GetAccessToken;
use crate::{api::soundcloud::auth_client::login_to_sc, credentials_manager::CredentialsOutputEvent, global_state::{RoudyData, RoudyDataMessage}};

#[derive(PartialEq)]
pub enum CredentialsListenerMessage {
    None,
    NewTokenReceived
}

pub async fn credentials_listener(
    credentials_receiver: &mut Receiver<CredentialsOutputEvent>,
    roudy_data: &mut RoudyData,
    get_access_token: &mut GetAccessToken,
    csrf_token: &mut Option<oauth2::CsrfToken>,
    access_token: &mut Option<String>,
    error_state: &mut ErrorState,
    ) -> CredentialsListenerMessage {
    let mut message = CredentialsListenerMessage::None;
    if let Ok(cred_message) = credentials_receiver.try_recv() {
        match cred_message {
            CredentialsOutputEvent::PromptLogin => {
                match login_to_sc().await {
                    Ok(soundcloud_auth) => {
                        RoudyData::update(
                            roudy_data,
                            RoudyDataMessage::SetLoginURL(soundcloud_auth.auth_url),
                        );
                        *get_access_token = Some(soundcloud_auth.get_access_token);
                        *csrf_token = Some(soundcloud_auth.csrf_token);
                    }
                    Err(e) => {
                        panic!("Failed to make login url: \n {e}")
                    }
                };
            }
            CredentialsOutputEvent::AccessToken(token) => {
                *access_token = Some(token);
                message = CredentialsListenerMessage::NewTokenReceived;
            }
            CredentialsOutputEvent::Error(message) => {
                ErrorState::update(error_state, ErrorMessage::CredentialsError(message));
            }
        }
    }
    message
}