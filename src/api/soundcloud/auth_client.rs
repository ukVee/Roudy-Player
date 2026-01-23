use dotenv::var;
use oauth2::{
    AuthType::RequestBody,
    AuthUrl,
    AuthorizationCode,
    ClientId,
    ClientSecret,
    CsrfToken,
    PkceCodeChallenge,
    RedirectUrl,
    Scope,
    TokenUrl,
    basic::BasicClient,
};
use crate::types::SoundCloudAuth;

pub async fn login_to_sc() -> anyhow::Result<SoundCloudAuth, anyhow::Error> {
    let client_id = var("CLIENT_ID")?;
    let partial_uri = var("REDIRECT_URI")?;
    let client_secret = var("CLIENT_SECRET")?;

    let client = BasicClient::new(ClientId::new(client_id))
        .set_client_secret(ClientSecret::new(client_secret))
        .set_auth_type(RequestBody)
        .set_auth_uri(AuthUrl::new(
            "https://secure.soundcloud.com/authorize".to_string(),
        )?)
        .set_token_uri(TokenUrl::new(
            "https://secure.soundcloud.com/oauth/token".to_string(),
        )?)
        
        .set_redirect_uri(RedirectUrl::new(partial_uri)?);

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read".to_string()))
        .add_scope(Scope::new("write".to_string()))
        .set_pkce_challenge(pkce_challenge)
        
        .url();
    let http_client = oauth2::reqwest::ClientBuilder::new()
        .redirect(oauth2::reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    Ok(SoundCloudAuth {
        csrf_token,
        auth_url,
        get_access_token: Box::new(move |code: String| {
            Box::pin(async move {
                let token = client
                    .exchange_code(AuthorizationCode::new(code))
                    
                    .set_pkce_verifier(pkce_verifier)
                    
                    .request_async(&http_client)
                    
                    .await?;
                    
                Ok(token)
            })
        }),
    })
}
