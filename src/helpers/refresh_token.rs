use dotenv::var;
use oauth2::{ClientId, ClientSecret, RefreshToken, TokenUrl, basic::BasicClient};
use std::fs::File;
use oauth2::TokenResponse;
use std::io::Write;
use crate::types::AuthCredentials;

pub fn save_token_to_file(auth_token: oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType >) -> Option<String> {
    let file_path = "auth_credentials.json";
    let auth_credentials = AuthCredentials {
        access_token: auth_token.access_token().secret().clone(),
        refresh_token: auth_token
            .refresh_token()
            .expect("Failed to get access token.")
            .secret()
            .clone(),
        expires_at: format!(
            "{:?}",
            &auth_token
                .expires_in()
                .expect("Failed to get auth expiration date.")
                .as_secs()
                + std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Should get time.")
                    .as_secs()
        ),
    };
    let json_auth_cred = serde_json::to_string_pretty(&auth_credentials)
        .expect("Failed to seralize to json");
    let mut file = File::create("auth_credentials.json")
        .expect("failed to create file.");
    match file.write_all(json_auth_cred.as_bytes()) {
       Ok(_) => Some(file_path.to_string()),
       Err(_) => None
    }
}

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
    let _ = save_token_to_file(new_token);
    Ok(())
}
