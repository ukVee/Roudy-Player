use oauth2::{
    CsrfToken,
    url::Url,
};
use ratatui::crossterm::event::KeyEvent;
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    pin::Pin,
};

pub enum PollEvent {
    Input(KeyEvent),
    Tick,
}


pub struct SoundCloudAuth {
    pub csrf_token: CsrfToken,
    pub auth_url: Url,
    pub get_access_token: Box<
        dyn FnOnce(String) -> Pin<Box<dyn Future<Output = Result<
            oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType > ,
            anyhow::Error
        >> + Send>>
        + Send
    >,
}


pub type GetAccessToken = Option<Box<
        dyn FnOnce(String) -> Pin<Box<dyn Future<Output = Result<
            oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType > ,
            anyhow::Error
        >> + Send>>
        + Send
    >>;



pub enum ServerEvent {
    Shutdown,
    Url(String),
}

pub struct QueryParams {
    pub authorization_code: Option<String>,
    pub csrf_state: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthCredentials {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: String,

}