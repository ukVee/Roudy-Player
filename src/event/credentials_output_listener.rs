use crate::global_state::{ErrorMessage, ErrorState};
use crate::types::GetAccessToken;
use crate::{
    api::soundcloud::auth_client::login_to_sc,
    auth::credentials_manager::CredentialsOutputEvent,
    global_state::{RoudyData, RoudyDataMessage},
};


pub async fn credentials_listener(
    msg: CredentialsOutputEvent,
    roudy_data: &mut RoudyData,
    get_access_token: &mut GetAccessToken,
    csrf_token: &mut Option<oauth2::CsrfToken>,
    access_token: &mut Option<String>,
    error_state: &mut ErrorState,) -> bool {
    
    let mut new_token = false;
    match msg {
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
            new_token = true;
        }
        CredentialsOutputEvent::Error(message) => {
            ErrorState::update(error_state, ErrorMessage::CredentialsError(message));
        }
    };
    new_token
}
