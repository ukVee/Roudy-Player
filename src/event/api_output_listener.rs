use tokio::sync::mpsc::Receiver;

use crate::{api::request_handler::ApiOutput, global_state::{ApiData, ApiDataMessage, ErrorMessage, ErrorState}};

pub fn api_listener(api_data_receiver: &mut Option<Receiver<ApiOutput>>, api_data: &mut ApiData, error_state: &mut ErrorState ) {
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
                    ApiData::update(api_data, ApiDataMessage::PlaylistsFetched(data));
                }
            }
        }
    }
}