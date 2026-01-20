use oauth2::url::Url;


pub struct GlobalState {
    pub logged_in: bool,
    pub login_url: Option<Url>,
}

impl GlobalState {
    pub fn new() -> GlobalState {
        Self {
            logged_in: false,
            login_url: None
        }
    }
}

pub struct ErrorState {
    pub failed_to_parse_state_param: bool,
    pub failed_to_parse_code_param: bool,
    pub csrf_token_does_not_match: bool,
    pub failed_to_shutdown_server: bool,
}

impl ErrorState {
    pub fn new() -> ErrorState {
        Self {
            failed_to_parse_state_param: false,
            failed_to_parse_code_param: false,
            csrf_token_does_not_match: false,
            failed_to_shutdown_server: false,
        }
    }
}