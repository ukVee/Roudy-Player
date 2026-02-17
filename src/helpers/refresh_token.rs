use dotenv::var;
use oauth2::{ClientId, ClientSecret, RefreshToken, TokenUrl, basic::BasicClient};
use std::fs::File;

use std::io::Write;
use crate::types::AuthCredentials;

pub async fn refresh_auth_token() -> anyhow::Result<()> {
    let client_id = var("CLIENT_ID")?;
    let client_secret = var("CLIENT_SECRET")?;
    let auth_file = File::open("auth_credentials.json")?;
    let jsonfile: AuthCredentials = serde_json::from_reader(auth_file)?;

    let client = BasicClient::new(ClientId::new(client_id))
        .set_client_secret(ClientSecret::new(client_secret))
        .set_auth_type(oauth2::AuthType::RequestBody)
        .set_token_uri(TokenUrl::new(
            "https://secure.soundcloud.com/oauth/token".to_string(),
        )?);
    let http_client = oauth2::reqwest::ClientBuilder::new()
        .redirect(oauth2::reqwest::redirect::Policy::none())
        .build()?;

    let new_token = client
        .exchange_refresh_token(&RefreshToken::new(jsonfile.refresh_token))
        .request_async(&http_client)
        .await?;
    // let _ = save_token_to_file(new_token);
    Ok(())
}
