use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{Receiver, Sender};
use oauth2::{StandardTokenResponse,EmptyExtraTokenFields};
use oauth2::basic::BasicTokenType;
use oauth2::TokenResponse;

use tokio::fs::File;
use crate::types::AuthCredentials;

pub enum CredentialsOutputEvent {
    AccessToken(String),
    NoAccessToken,
    Error(String),
    PromptLogin,
}

pub enum CredentialsEvent {
    Shutdown,
    RequestAccessToken,
    SaveToken(StandardTokenResponse<EmptyExtraTokenFields,BasicTokenType>)
}

pub struct CredentialsManager {
    pub cred_channels: (Sender<CredentialsEvent>, Receiver<CredentialsOutputEvent>),
}
impl CredentialsManager {
    pub async fn mount() -> Self {
        let path = "auth_credentials.json".to_string();
        let (cred_messenger, mut cred_rx) = tokio::sync::mpsc::channel::<CredentialsEvent>(32);
        let (cred_data_sender, cred_output_receiver) = tokio::sync::mpsc::channel::<CredentialsOutputEvent>(32);

        tokio::spawn( async move {
            let mut current_credentials: Option<AuthCredentials> = None;
            let mut first_time = true;
            loop {
                if let Ok(cred_event) = cred_rx.try_recv() {
                    match cred_event {
                        CredentialsEvent::Shutdown => {
                            cred_rx.close();
                            
                            break
                        }
                        CredentialsEvent::RequestAccessToken => {
                            match &current_credentials {
                                Some(creds) => {
                                    let _ = cred_data_sender.send(CredentialsOutputEvent::AccessToken(creds.access_token.clone())).await;
                                }
                                None => {
                                    let _ = cred_data_sender.send(CredentialsOutputEvent::NoAccessToken).await;
                                }
                            }
                        }
                        CredentialsEvent::SaveToken(token) => {
                            let new_creds = CredentialsManager::save_auth_info_to_file(token, &path).await;
                            let _ = cred_data_sender.send(CredentialsOutputEvent::AccessToken(new_creds.access_token.clone())).await;
                            current_credentials = Some(new_creds);
                        }
                    }
                }
                if current_credentials.is_none() {
                    
                    let auth_cred = CredentialsManager::get_auth_credentials_from_file(path.clone());
                    match auth_cred {
                        Ok(creds) => {
                            let access_token = creds.access_token.clone();
                            current_credentials = Some(creds);
                            let _ = cred_data_sender.send(CredentialsOutputEvent::AccessToken(access_token)).await;
                        }
                        Err(_) => {
                            if first_time {//only send the prompt login once
                                first_time = false;
                                let _ = cred_data_sender.send(CredentialsOutputEvent::PromptLogin).await;
                            }
                        }
                    }
                }
            }
        });

        Self {
            cred_channels: (cred_messenger, cred_output_receiver)
        }
    }

    fn get_auth_credentials_from_file(path: String) -> Result<AuthCredentials> {
        let file = std::fs::File::open(path)?;
        let json_data: AuthCredentials = serde_json::from_reader(file)?;
        Ok(json_data)
    }

    async fn save_auth_info_to_file(token: StandardTokenResponse<EmptyExtraTokenFields,BasicTokenType>, file_path: &String) -> AuthCredentials {
        let auth_cred = AuthCredentials {
            access_token: token.access_token().secret().clone(),
            refresh_token: token
                .refresh_token()
                .expect("Failed to get access token.")
                .secret()
                .clone(),
            expires_at: format!(
                "{:?}",
                &token
                    .expires_in()
                    .expect("Failed to get auth expiration date.")
                    .as_secs()
                    + std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("Should get time.")
                        .as_secs()),
        };
        let json_auth_cred = serde_json::to_string_pretty(&auth_cred)
            .expect("Failed to seralize to json");

        let mut file = File::create(file_path).await
            .expect("Failed to create file.");

        match file.write_all(json_auth_cred.as_bytes()).await {
            Ok(_) => {},
            Err(_) => {}
        }

        auth_cred
    }

}