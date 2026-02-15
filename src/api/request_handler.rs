use crate::{api::soundcloud::profile::get_profile, global_state::RoudyData, types::AuthCredentials};
use anyhow::Result;
use std::fs;
use tokio::sync::mpsc::{Receiver, Sender};
pub enum ClientEvent {
    GetProfile,
    Shutdown,
}

pub fn validate_path(path: Option<String>) -> String {
    match path {
        Some(path) => {
            path.to_string()
        },
        None => {//default config location
            "~/.config/roudy/auth_credentials.json".to_string()
        }
    }
}

pub async fn mount_client_request_handler(roudy_data: &RoudyData) -> Result<(Sender<ClientEvent>, Receiver<String>)> {
    let path= roudy_data.token_path.clone();
    let valed_path = validate_path(path);
    request_handler(&valed_path).await
}

pub async fn request_handler(token_path: &String) -> Result<(Sender<ClientEvent>, Receiver<String>)> {
    let client = reqwest::Client::new();
    let file = fs::File::open(token_path).expect("Failed to find token file");
    let auth: AuthCredentials = serde_json::from_reader(file).expect("Failed to parse file.");

    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<ClientEvent>(32);
    let (data_tx, mut data_rx) = tokio::sync::mpsc::channel::<>(32);

    tokio::spawn(async move {
        loop {
            if let Ok(event) = event_rx.try_recv() {
                match event {
                    ClientEvent::GetProfile => {
                        let profile_data = get_profile(&client, &auth.access_token).await;
                        match profile_data {
                            Ok(data) => {
                                let _ = data_tx.send(data).await;
                            }
                            Err(e) => {
                                let _ = data_tx.send(e.to_string());
                            }
                        }
                    }
                    ClientEvent::Shutdown => {
                        event_rx.close();
                        break
                    }
                };
            };
        }
    });
    Ok((event_tx, data_rx))
}
