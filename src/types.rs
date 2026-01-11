use oauth2::{
    url::Url,
    PkceCodeVerifier,
    CsrfToken,
};
use ratatui::crossterm::event::KeyEvent;

pub enum PollEvent {
    Input(KeyEvent),
    Tick,
}


pub struct SoundCloudAuth {
    pub auth_url: Url,
    pub pkce_verifier: PkceCodeVerifier,
    pub csrf_token: CsrfToken,
}

pub enum ServerEvent {
    Shutdown,
    Url(String),
}