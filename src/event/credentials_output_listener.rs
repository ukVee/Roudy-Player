use tokio::sync::mpsc::Receiver;
use oauth2::StandardTokenResponse;
use oauth2::EmptyExtraTokenFields;
use std::pin::Pin;
use oauth2::basic::BasicTokenType;
use crate::types::GetAccessToken;
use crate::{api::soundcloud::auth_client::login_to_sc, credentials_manager::CredentialsOutputEvent, global_state::{RoudyData, RoudyDataMessage}};



pub async fn mount_credentials_listener(
    credentials_receiver: &mut Receiver<CredentialsOutputEvent>,
    roudy_data: &mut RoudyData,
    get_access_token: &mut GetAccessToken,
    csrf_token: Option<oauth2::CsrfToken>,
    access_token: Option<String>,
    ) {
    if let Ok(cred_message) = credentials_receiver.try_recv() {
        match cred_message {
            CredentialsOutputEvent::PromptLogin => {
                match login_to_sc().await {
                    Ok(soundcloud_auth) => {
                        RoudyData::update(
                            roudy_data,
                            RoudyDataMessage::SetLoginURL(soundcloud_auth.auth_url),
                        );
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
}