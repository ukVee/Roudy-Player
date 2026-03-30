use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::Arc;
use ratatui::crossterm::event::KeyCode;
use tokio::sync::mpsc::{Sender};

use crate::event::keybind::audio_keybinds::listen_for_audio_keybinds;
use crate::{
    api::request_handler::ClientEvent,
    event::keybind::homepage_keybinds::listen_for_homepage_binds,
    global_state::{ApiData, Roudy, RoudyMessage, SelectedTab},
    types::PollEvent,
};

pub async fn keypress_listener(
    msg: PollEvent,
    req_api_data: &Option<Sender<ClientEvent>>,
    global_state: &mut Roudy,
    api_data: &mut ApiData,
    paused: Arc<AtomicBool>,
    volume: Arc<AtomicU32>,
) -> bool {
    let mut shutdown_flag = false;

    match msg {
        PollEvent::Input(key) => {
            if key.code == KeyCode::Char('q') {
                shutdown_flag = true;
            } else if key.code == KeyCode::Tab && global_state.logged_in {
                match global_state.selected_tab {
                    SelectedTab::Home => {
                        Roudy::update(global_state, RoudyMessage::ChangeTab(SelectedTab::Profile));
                    }
                    SelectedTab::Profile => {
                        Roudy::update(
                            global_state,
                            RoudyMessage::ChangeTab(SelectedTab::ErrorStatus),
                        );
                    }
                    SelectedTab::ErrorStatus => {
                        Roudy::update(global_state, RoudyMessage::ChangeTab(SelectedTab::Test));
                    }
                    SelectedTab::Test => {
                        Roudy::update(global_state, RoudyMessage::ChangeTab(SelectedTab::Home));
                    }
                };

                match global_state.selected_tab {
                    SelectedTab::Home => {
                        if let Some(sender) = req_api_data.as_ref() {
                            let _ = sender.send(ClientEvent::GetPlaylists).await;
                        }
                    }
                    SelectedTab::Profile => {
                        if let Some(sender) = req_api_data.as_ref() {
                            let _ = sender.send(ClientEvent::GetProfile).await;
                        }
                    }
                    SelectedTab::ErrorStatus => {}
                    SelectedTab::Test => {
                        if let Some(sender) = req_api_data.as_ref() {
                            if let Some(tracks) = &api_data.playlist_tracks {
                                let _ = sender
                                    .send(ClientEvent::GetTrackMetadata(
                                        tracks[global_state.homepage_tracks_scroll_offset as usize]
                                            .id,
                                    ))
                                    .await;
                            }
                        }
                    }
                }
            }
            if global_state.selected_tab == SelectedTab::Home {
                listen_for_homepage_binds(key, &req_api_data, global_state, api_data).await;
            }
            listen_for_audio_keybinds(key, paused, volume);
        }
    }
    shutdown_flag
}
