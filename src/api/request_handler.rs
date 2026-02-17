use crate::{api::soundcloud::{playlist::get_playlists, profile::get_profile}, };
use tokio::sync::mpsc::{Receiver, Sender};
pub enum ClientEvent {
    GetProfile,
    GetPlaylists,
    Shutdown,
}
pub enum ApiOutput {
    Profile(String),
    Playlists(String),
    Error(String),
}

pub struct ApiRequestHandler {
    pub api_req_handler_messenger: Sender<ClientEvent>,
    pub api_data_receiver: Receiver<ApiOutput>,
}

impl ApiRequestHandler {
    pub async fn mount(access_token: String) -> Self {
    let client = reqwest::Client::new();

    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<ClientEvent>(32);
    let (data_tx, data_rx) = tokio::sync::mpsc::channel::<ApiOutput>(32);

    tokio::spawn(async move {
        loop {
            if let Ok(event) = event_rx.try_recv() {
                match event {
                    ClientEvent::GetProfile => {
                        let profile_data = get_profile(&client, &access_token).await;
                        match profile_data {
                            Ok(data) => {
                                let _ = data_tx.send(ApiOutput::Profile(data)).await;
                            }
                            Err(e) => {
                                let _ = data_tx.send(ApiOutput::Error(e.to_string()));
                            }
                        }
                    }
                    ClientEvent::GetPlaylists => {
                        let playlist_data = get_playlists(&client, &access_token).await;
                        match playlist_data {
                            Ok(data) => {
                                let _ = data_tx.send(ApiOutput::Playlists(data)).await;
                            }
                            Err(e) => {
                                let _ = data_tx.send(ApiOutput::Error(e.to_string())).await;
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
    Self {
        api_data_receiver: data_rx,
        api_req_handler_messenger: event_tx
    }
}

}


