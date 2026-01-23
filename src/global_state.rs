use oauth2::url::Url;

pub enum RoudyMessage {
    Login,
} 
pub struct Roudy {
    pub logged_in: bool,

}
impl Roudy {
    pub fn new() -> Self {
        Self {
            logged_in: false,
        }
    }

    pub fn update(model: &mut Roudy, msg: RoudyMessage) -> Option<RoudyMessage> {
        match msg {
            RoudyMessage::Login => {
                model.logged_in = true;
            }
        }
        None
    }
}


pub enum RoudyDataMessage {
    SetLoginURL(Url),
    SetTokenPath(String)
}
pub struct RoudyData {
    login_url: Option<Url>,    
    token_path: Option<String>,
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


pub enum ErrorMessage {
    FailedCodeParamParse,
    CSRFTokenDoesntMatch,
    FailedServerShutdown,
    FailedCSRFParamParse,
}

pub struct ErrorState {
    pub failed_to_parse_code_param: bool,
    pub csrf_token_does_not_match: bool,
    pub failed_to_shutdown_server: bool,
    pub failed_to_parse_csrf_param: bool,
}

impl ErrorState {
    pub fn new() -> ErrorState {
        Self {
            failed_to_parse_code_param: false,
            csrf_token_does_not_match: false,
            failed_to_shutdown_server: false,
            failed_to_parse_csrf_param: false,
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
        }
    }
}