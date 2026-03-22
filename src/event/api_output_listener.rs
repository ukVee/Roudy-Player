
use crate::{api::request_handler::ApiOutput, audio::audio_handler::{AudioCommand, AudioHandler, AudioMessage}, global_state::{ApiData, ApiDataMessage, ErrorMessage, ErrorState, Roudy, RoudyMessage}};

pub fn api_listener(api_data_receiver: &mut Option<tokio::sync::mpsc::Receiver<ApiOutput>>, audio_receiver: &mut std::sync::mpsc::Sender<AudioCommand>, audio_handler: &mut AudioHandler, global_state: &mut Roudy, api_data: &mut ApiData, error_state: &mut ErrorState ) {
    if let Some(rx) = api_data_receiver.as_mut() {
        if let Ok(api_event) = rx.try_recv() {
            match api_event {
                ApiOutput::Error(message) => {
                    ErrorState::update(error_state, ErrorMessage::ApiError(message));
                }
                ApiOutput::Profile(data) => {
                    ApiData::update(api_data, ApiDataMessage::ProfileFetched(data));
                }
                ApiOutput::Playlists(data) => {
                    Roudy::update(global_state, RoudyMessage::HOMEPAGEUpdatePlaylistCount(data.len()));
                    ApiData::update(api_data, ApiDataMessage::PlaylistsFetched(data));
                }
                ApiOutput::PlaylistTracks(data) => {
                    Roudy::update(global_state, RoudyMessage::HOMEPAGEUpdateTracksCount(data.len()));
                    ApiData::update(api_data, ApiDataMessage::PlaylistTracksFetched(data));
                }
                ApiOutput::TrackStream(data) => {
                    ApiData::update(api_data, ApiDataMessage::TrackStreamFetched(data.clone()));
                    let _ = audio_receiver.send(AudioCommand::HlsReceived(data.clone()));
                }
                ApiOutput::TrackMetadata(data) => {
                    ApiData::update(api_data, ApiDataMessage::TrackMetadataFetched(data));
                }
                ApiOutput::TrackMediaPlaylist(data) => {
                    AudioHandler::update(audio_handler, AudioMessage::StoreMediaPlaylist(data));
                }
            }
        }
    }
}