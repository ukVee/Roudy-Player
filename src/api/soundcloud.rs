use std::error;
use dotenv::var;

use oauth2::{
    AuthUrl,
    ClientId,
    ClientSecret,
    CsrfToken,
    PkceCodeChallenge,
    RedirectUrl,
    Scope,
    TokenUrl
};
use oauth2::basic::BasicClient;

use crate::types::SoundCloudAuth;

pub async fn login_to_sc() -> Result<SoundCloudAuth, Box<dyn error::Error>> {
    let client_id = var("CLIENT_ID")?;
    let redirect_uri = var("REDIRECT_URI")?;
    let client_secret = var("CLIENT_SECRET")?;
    
    let client = BasicClient::new(ClientId::new(client_id))
        .set_client_secret(ClientSecret::new(client_secret))
        .set_auth_uri(AuthUrl::new("https://secure.soundcloud.com/authorize".to_string())?)
        .set_token_uri(TokenUrl::new("https://secure.soundcloud.com/oauth/token".to_string())?)
        .set_redirect_uri(RedirectUrl::new(redirect_uri)?);
        
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read".to_string()))
        .add_scope(Scope::new("write".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    // let http_client = reqwest::ClientBuilder::new()
    //     .redirect(reqwest::redirect::Policy::none())
    //     .build()
    //     .expect("http_client should build");

    Ok(SoundCloudAuth {
        auth_url,
        pkce_verifier,
        csrf_token,

    })
}