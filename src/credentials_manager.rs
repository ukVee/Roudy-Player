use anyhow::Result;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::types::AuthCredentials;

enum CredentialsOutputEvent {
    AccessToken(String),
    NoAccessToken,
    Error(String)
}

enum CredentialsEvent {
    Shutdown,
    RequestAccessToken
}

struct CredentialsManager {
    auth_credentials_path: Option<String>,
    cred_messenger: Option<Sender<CredentialsEvent>>,
    cred_output_receiver: Option<Receiver<CredentialsOutputEvent>>
}
impl CredentialsManager {
    fn new() -> Self {
        Self {
            auth_credentials_path: None,
            cred_messenger: None,
            cred_output_receiver: None,
        }
    }
    pub async fn mount(credman: &mut Self) {
        let path = CredentialsManager::get_path(&credman);
        let (cred_messenger, mut cred_rx) = tokio::sync::mpsc::channel::<CredentialsEvent>(32);
        let (cred_data_sender, cred_output_receiver) = tokio::sync::mpsc::channel::<CredentialsOutputEvent>(32);

        tokio::spawn( async move {
            let mut current_access_token: Option<String> = None;
            loop {
                if let Ok(cred_event) = cred_rx.try_recv() {
                    match cred_event {
                        CredentialsEvent::Shutdown => {
                            cred_rx.close();
                            break
                        }
                        CredentialsEvent::RequestAccessToken => {
                            match &current_access_token {
                                Some(token) => {
                                    let _ = cred_data_sender.send(CredentialsOutputEvent::AccessToken(token.clone())).await;
                                }
                                None => {
                                    let _ = cred_data_sender.send(CredentialsOutputEvent::NoAccessToken).await;
                                }
                            }
                        }
                    }
                }
                if current_access_token.is_none() {
                    let auth_cred = CredentialsManager::get_auth_credentials_from_file(path.clone());
                    match auth_cred {
                        Ok(creds) => {
                            current_access_token = Some(creds.access_token);
                        }
                        Err(e) => {
                            let _ = cred_data_sender.send(CredentialsOutputEvent::Error(e.to_string())).await;
                        }
                    }
                }
            }
        });

        CredentialsManager::mount_messenger(credman, cred_messenger);
        CredentialsManager::mount_output_receiver(credman, cred_output_receiver);
    }

    fn mount_messenger(credman: &mut Self, cred_messenger: Sender<CredentialsEvent>) {
        credman.cred_messenger = Some(cred_messenger);
    }
    fn mount_output_receiver(credman: &mut Self, cred_output_receiver: Receiver<CredentialsOutputEvent>) {
        credman.cred_output_receiver = Some(cred_output_receiver);
    }

    fn get_path(credman: &Self) -> String {
        match &credman.auth_credentials_path {
            Some(path) => path.clone(),
            None => "~/projects/roudy/auth_credentials.json".to_string(),
        }
    }

    fn update_path(credman: &mut Self, path:&String){
        credman.auth_credentials_path = Some(path.clone().to_string());
    }

    fn get_auth_credentials_from_file(path: String) -> Result<AuthCredentials> {
        let file = std::fs::File::open(path)?;
        let json_data: AuthCredentials = serde_json::from_reader(file)?;
        Ok(json_data)
    }

    

}