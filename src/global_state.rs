use oauth2::url::Url;
use tokio::sync::mpsc::{Receiver, Sender};

pub enum RoudyMessage {
    Login,
    ChangeTab(usize),
} 
pub struct Roudy {
    pub logged_in: bool,
    pub selected_tab: usize,
}
impl Roudy {
    pub fn new() -> Self {
        Self {
            logged_in: false,
            selected_tab: 0,
        }
    }

    pub fn update(model: &mut Roudy, msg: RoudyMessage) -> Option<RoudyMessage> {
        match msg {
            RoudyMessage::Login => {
                model.logged_in = true;
            },
            RoudyMessage::ChangeTab(new_tab) => {
                model.selected_tab = new_tab;
            },
        }
        None
    }
}


pub enum RoudyDataMessage {
    SetLoginURL(Url),
    SetTokenPath(String)
}
pub struct RoudyData {
    pub login_url: Option<Url>,    
    pub token_path: Option<String>,
}
impl RoudyData {
    pub fn new() -> Self {
        Self {
            login_url: None,
            token_path: None,
        }
    }
    pub fn update(model: &mut RoudyData, msg: RoudyDataMessage) -> Option<RoudyDataMessage> {
        match msg {
            RoudyDataMessage::SetLoginURL(url) => {
                model.login_url = Some(url);
            }
            RoudyDataMessage::SetTokenPath(path) => {
                model.token_path = Some(path);
            }
        }
        None
    }
}

pub enum ApiDataMessage {
    ProfileFetched(String),
    PlaylistsFetched(String),
}
pub struct ApiData {
    pub profile: Option<String>,
    pub playlists: Option<String>,
}
impl ApiData {
    pub fn new() -> Self {
        Self {
            profile: None,
            playlists: None,
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
        }
        None
    }
}


pub enum ErrorMessage {
    FailedCodeParamParse,
    CSRFTokenDoesntMatch,
    FailedServerShutdown,
    FailedCSRFParamParse,
    FailedMountClientRequestHandler,
    ApiError(String),
}

pub struct ErrorState {
    pub failed_to_parse_code_param: bool,
    pub csrf_token_does_not_match: bool,
    pub failed_to_shutdown_server: bool,
    pub failed_to_parse_csrf_param: bool,
    pub failed_to_mount_client_request_handler: bool,
    pub api_error_log: Vec<String>,
}

impl ErrorState {
    pub fn new() -> Self {
        Self {
            failed_to_parse_code_param: false,
            csrf_token_does_not_match: false,
            failed_to_shutdown_server: false,
            failed_to_parse_csrf_param: false,
            failed_to_mount_client_request_handler: false,
            api_error_log: Vec::new(),
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
            ErrorMessage::FailedMountClientRequestHandler => {
                error_model.failed_to_mount_client_request_handler = true;
            }
            ErrorMessage::ApiError(message) => {
                error_model.api_error_log.push(message);
            }
        }
    }
}