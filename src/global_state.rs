use oauth2::url::Url;

use crate::api::soundcloud::{playlist::APIPlaylist, playlist_tracks::APIPlaylistTracks, profile::APIProfile};
pub enum RoudyMessage {
    Login,
    ChangeTab(usize),
    HOMEPAGEUpdatePlaylistScrollOffset(i32),
    HOMEPAGEUpdatePlaylistCount(usize),
    HOMEPAGEChangeSubpage(i32),
    HOMEPAGEUpdateTracksScrollOffset(i32),
    HOMEPAGEUpdateTracksCount(usize),
} 
pub struct Roudy {
    pub logged_in: bool,
    pub selected_tab: usize,
    pub homepage_playlist_scroll_offset: i32,
    pub homepage_playlist_count: usize,
    pub homepage_subpage: i32, // 0 = playlist carousel 1 = playlist_tracks carousel
    pub homepage_tracks_scroll_offset: i32,
    pub homepage_tracks_count: usize,
}

impl Roudy {
    pub fn new() -> Self {
        Self {
            logged_in: false,
            selected_tab: 0,
            homepage_playlist_scroll_offset: 0,
            homepage_playlist_count: 0,
            homepage_subpage: 0,
            homepage_tracks_scroll_offset: 0,
            homepage_tracks_count: 0,
        }
    }

    pub fn update(model: &mut Roudy, msg: RoudyMessage) -> Option<RoudyMessage> {
        match msg {
            RoudyMessage::Login => {
                model.logged_in = true;
            }
            RoudyMessage::ChangeTab(new_tab) => {
                model.selected_tab = new_tab;
            }
            RoudyMessage::HOMEPAGEUpdatePlaylistScrollOffset(offset) => {
                model.homepage_playlist_scroll_offset = offset;
            }
            RoudyMessage::HOMEPAGEUpdatePlaylistCount(count) => {
                model.homepage_playlist_count = count;
            }
            RoudyMessage::HOMEPAGEChangeSubpage(new_page) => {
                model.homepage_subpage = new_page;
            }
            RoudyMessage::HOMEPAGEUpdateTracksScrollOffset(offset) => {
                model.homepage_tracks_scroll_offset = offset;
            }
            RoudyMessage::HOMEPAGEUpdateTracksCount(count) => {
                model.homepage_tracks_count = count;
            }
        }
        None
    }
}


pub enum RoudyDataMessage {
    SetLoginURL(Url),
}
pub struct RoudyData {
    pub login_url: Option<Url>,    
}
impl RoudyData {
    pub fn new() -> Self {
        Self {
            login_url: None,
        }
    }
    pub fn update(model: &mut RoudyData, msg: RoudyDataMessage) -> Option<RoudyDataMessage> {
        match msg {
            RoudyDataMessage::SetLoginURL(url) => {
                model.login_url = Some(url);
            }
        }
        None
    }
}

pub enum ApiDataMessage {
    ProfileFetched(APIProfile),
    PlaylistsFetched(Vec<APIPlaylist>),
    PlaylistTracksFetched(Vec<APIPlaylistTracks>),
}
pub struct ApiData {
    pub profile: Option<APIProfile>,
    pub playlists: Option<Vec<APIPlaylist>>,
    pub playlist_tracks: Option<Vec<APIPlaylistTracks>>,
}
impl ApiData {
    pub fn new() -> Self {
        Self {
            profile: None,
            playlists: None,
            playlist_tracks: None,
        }
    }
    pub fn update(model: &mut Self, msg: ApiDataMessage) -> Option<ApiDataMessage> {
        match msg {
            ApiDataMessage::ProfileFetched(data) => {
                model.profile = Some(data);
            }
            ApiDataMessage::PlaylistsFetched(data) => {
                model.playlists = Some(data);
            }
            ApiDataMessage::PlaylistTracksFetched(data) => {
                model.playlist_tracks = Some(data);
            }
        }
        None
    }
}


pub enum ErrorMessage {
    FailedCodeParamParse,
    CSRFTokenDoesntMatch,
    FailedServerShutdown,
    FailedCSRFParamParse,
    FailedMountApiRequestHandler,
    ApiError(String),
    CredentialsError(String),
}

pub struct ErrorState {
    pub failed_to_parse_code_param: bool,
    pub csrf_token_does_not_match: bool,
    pub failed_to_shutdown_server: bool,
    pub failed_to_parse_csrf_param: bool,
    pub failed_to_mount_api_request_handler: bool,
    pub api_error_log: Vec<String>,
    pub credentials_error_log: Vec<String>,
}

impl ErrorState {
    pub fn new() -> Self {
        Self {
            failed_to_parse_code_param: false,
            csrf_token_does_not_match: false,
            failed_to_shutdown_server: false,
            failed_to_parse_csrf_param: false,
            failed_to_mount_api_request_handler: false,
            api_error_log: Vec::new(),
            credentials_error_log: Vec::new(),
        }
    }

    pub fn update(error_model: &mut ErrorState, msg: ErrorMessage)  {
        match msg {
            ErrorMessage::FailedCodeParamParse=> {
                error_model.failed_to_parse_code_param = true;
            }
            ErrorMessage::CSRFTokenDoesntMatch=> {
                error_model.csrf_token_does_not_match = true;
            }
            ErrorMessage::FailedServerShutdown=> {
                error_model.failed_to_shutdown_server = true;
            }
            ErrorMessage::FailedCSRFParamParse => {
                error_model.failed_to_parse_csrf_param = true;
            }
            ErrorMessage::FailedMountApiRequestHandler => {
                error_model.failed_to_mount_api_request_handler = true;
            }
            ErrorMessage::ApiError(message) => {
                error_model.api_error_log.push(message);
            }
            ErrorMessage::CredentialsError(message) => {
                error_model.credentials_error_log.push(message);
            }
        }
    }
}