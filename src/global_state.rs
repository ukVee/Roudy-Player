
pub struct GlobalState {
    pub soundcloud_url: Option<String>,
    pub logged_in: bool,
    pub authorization_code: Option<String>,
}

impl GlobalState {
    pub fn new() -> GlobalState {
        Self {
            soundcloud_url: None,
            logged_in: false,
            authorization_code: None,
        }
    }
}

pub struct ErrorState {
    pub failed_to_parse_params: bool,
}

impl ErrorState {
    pub fn new() -> ErrorState {
        Self {
            failed_to_parse_params: false,
        }
    }
}