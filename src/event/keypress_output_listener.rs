use ratatui::crossterm::event::KeyCode;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{api::request_handler::{ApiOutput, ClientEvent}, credentials_manager::{CredentialsEvent, CredentialsOutputEvent}, global_state::{Roudy, RoudyMessage}, types::{PollEvent, ServerEvent}};

#[derive(PartialEq)]
pub enum KeypressListenerStatus {
    Shutdown,
    Continue,
}

pub async fn keypress_listener(
    keybind_receiver: &mut Receiver<PollEvent>,
    server_receiver: &mut Receiver<ServerEvent>,
    api_data_receiver: &mut Option<Receiver<ApiOutput>>,
    req_api_data: &Option<Sender<ClientEvent>>,
    credentials_receiver: &mut Receiver<CredentialsOutputEvent>,
    credentials_messenger: &Sender<CredentialsEvent>,
    shutdown_auth_server: & Sender<()>,
    global_state: &mut Roudy,
    ) -> KeypressListenerStatus {
    const PAGES: usize = 3;
    let mut status: KeypressListenerStatus = KeypressListenerStatus::Continue;

    if let Ok(key_pressed) = keybind_receiver.try_recv() {
        match key_pressed {
            PollEvent::Input(key) => {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    keybind_receiver.close();
                    server_receiver.close();
                    if let Some(rx) = api_data_receiver.as_mut() {
                        rx.close();
                    }
                    if let Some(tx) = req_api_data {
                        let _ = tx.send(ClientEvent::Shutdown).await;
                    }
                    credentials_receiver.close();
                    let _ = credentials_messenger.send(CredentialsEvent::Shutdown).await;
                    
                    let _ = shutdown_auth_server.send(()).await;
                    
                    
                    status = KeypressListenerStatus::Shutdown;
                } else if key.code == KeyCode::Tab && global_state.logged_in {
                    let mut new_tab = global_state.selected_tab + 1;
                    if new_tab >= PAGES {
                        new_tab = 0;
                    }
                    Roudy::update(global_state, RoudyMessage::ChangeTab(new_tab));
                    match new_tab {
                        0 => {
                            if let Some(sender) = req_api_data.as_ref() {
                                let _ = sender.send(ClientEvent::GetPlaylists).await;
                            }
                        }
                        1 => {
                            if let Some(sender) = req_api_data.as_ref() {
                                let _ = sender.send(ClientEvent::GetProfile).await;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    status
}