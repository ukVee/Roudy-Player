use std::fs;
use crate::{api::soundcloud::profile::get_profile, types::AuthCredentials};



pub async fn request_handler(req_type: &str, token_path: &str) -> String {
    let client = reqwest::Client::new();
    let file = fs::File::open(token_path)
        .expect("Failed to find token file");
    let auth: AuthCredentials = serde_json::from_reader(file)
        .expect("Failed to parse file.");

    match req_type {
        "profile" => {
            match get_profile(client, &auth.access_token).await {
                Ok(profile_data) => {
                    profile_data
                }
                Err(e) => {
                    e.to_string()
                }
            }
        }
        _ => "Invalid Req type".to_string()
    }
}