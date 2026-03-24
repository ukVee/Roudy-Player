use std::time::{SystemTime, UNIX_EPOCH};

use tokio::time::{Duration, Instant};

use crate::types::AuthCredentials;
use anyhow::Result;
use dotenv::var;
use oauth2::TokenResponse;
use oauth2::basic::BasicTokenType;
use oauth2::{ClientId, ClientSecret, RefreshToken, TokenUrl, basic::BasicClient};
use oauth2::{EmptyExtraTokenFields, StandardTokenResponse};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{Receiver, Sender};
pub enum CredentialsOutputEvent {
    AccessToken(String),
    Error(String),
    PromptLogin,
}

pub enum CredentialsEvent {
    Shutdown,
    SaveToken(StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>),
}

pub struct CredentialsManager {
    pub cred_channels: (Sender<CredentialsEvent>, Receiver<CredentialsOutputEvent>),
}
impl CredentialsManager {
    pub async fn mount() -> Self {
        let path = "auth_credentials.json".to_string();
        let (cred_messenger, mut cred_rx) = tokio::sync::mpsc::channel::<CredentialsEvent>(32);
        let (cred_data_sender, cred_output_receiver) = tokio::sync::mpsc::channel::<CredentialsOutputEvent>(32);
        
        tokio::spawn(async move {
            let mut current_credentials: Option<AuthCredentials> = None;
            let mut token_deadline: tokio::time::Instant;
            match CredentialsManager::get_auth_credentials_from_file(&path) {
                Ok(creds) => {
                    let access_token = creds.access_token.clone();
                    token_deadline = CredentialsManager::compute_deadline(creds.expires_at.clone());
                    current_credentials = Some(creds);
                    let _ = cred_data_sender
                        .send(CredentialsOutputEvent::AccessToken(access_token))
                        .await;
                }
                Err(_) => {
                    token_deadline = Instant::now() + Duration::from_secs(u32::MAX as u64);
                    let _ = cred_data_sender
                        .send(CredentialsOutputEvent::PromptLogin)
                        .await;
                }
            };
            loop {
                tokio::select! {
                    Some(cred_event) = cred_rx.recv() => {
                        match cred_event {
                            CredentialsEvent::Shutdown => {
                                cred_rx.close();
                                break;
                            }
                            CredentialsEvent::SaveToken(token) => {
                                let new_creds =
                                CredentialsManager::save_auth_info_to_file(token, &path).await;
                                token_deadline = CredentialsManager::compute_deadline(new_creds.expires_at.clone());
                                let _ = cred_data_sender
                                .send(CredentialsOutputEvent::AccessToken(
                                    new_creds.access_token.clone(),
                                ))
                                .await;
                                current_credentials = Some(new_creds);
                            }
                        };
                    },
                    _ = tokio::time::sleep_until(token_deadline) => {
                        if let Some(credentials) = current_credentials.as_ref() {
                            let expiration = credentials.expires_at;
                            
                            if AuthCredentials::is_token_expired(expiration) {
                                let new_token = CredentialsManager::refresh_auth_token(
                                    current_credentials
                                    .as_ref()
                                    .expect("should have credentials")
                                    .refresh_token
                                    .clone(),
                                ).await;
                                match new_token {
                                    Ok(token) => {
                                        let new_credentials =
                                        CredentialsManager::save_auth_info_to_file(token, &path).await;
                                        let access_token = new_credentials.access_token.clone();
                                        current_credentials = Some(new_credentials);
                                        let _ = cred_data_sender
                                        .send(CredentialsOutputEvent::AccessToken(access_token))
                                        .await;
                                    }
                                    Err(e) => {
                                        let _ = cred_data_sender
                                        .send(CredentialsOutputEvent::Error(e.to_string()))
                                        .await;
                                    }
                                };
                            };
                        }
                        
                    },
                };
            }
        });

        Self {
            cred_channels: (cred_messenger, cred_output_receiver),
        }
    }

    fn get_auth_credentials_from_file(path: &String) -> Result<AuthCredentials> {
        let file = std::fs::File::open(path)?;
        let json_data: AuthCredentials = serde_json::from_reader(file)?;
        Ok(json_data)
    }
    fn compute_deadline(expires_at: u64) -> tokio::time::Instant {
        let unix_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("should get time")
            .as_secs();
        
        if expires_at <= unix_now {
            tokio::time::Instant::now()
        } else {
            tokio::time::Instant::now() + Duration::from_secs(expires_at - unix_now)
        }
    }

    async fn save_auth_info_to_file(
        token: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
        file_path: &String,
    ) -> AuthCredentials {
        let auth_cred = AuthCredentials {
            access_token: token.access_token().secret().clone(),
            refresh_token: token
                .refresh_token()
                .expect("Failed to get access token.")
                .secret()
                .clone(),
            expires_at: 
                &token
                    .expires_in()
                    .expect("Failed to get auth expiration date.")
                    .as_secs()
                    + std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("Should get time.")
                        .as_secs()
            ,
        };
        let json_auth_cred =
            serde_json::to_string_pretty(&auth_cred).expect("Failed to seralize to json");

        let mut file = File::create(file_path)
            .await
            .expect("Failed to create file.");

        let _ = file.write_all(json_auth_cred.as_bytes()).await;

        auth_cred
    }

    async fn refresh_auth_token(
        refresh_token: String,
    ) -> anyhow::Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        let client_id = var("CLIENT_ID")?;
        let client_secret = var("CLIENT_SECRET")?;
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
            .exchange_refresh_token(&RefreshToken::new(refresh_token))
            .request_async(&http_client)
            .await?;

        Ok(new_token)
    }
}
