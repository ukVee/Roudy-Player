use reqwest::{Error};

#[derive(serde::Deserialize, Debug)]
pub struct APIProfile {
    #[serde(rename = "avatar_url")]
    pub _avatar_url: String,
    pub username: String,
    pub description: Option<String>,
    pub plan: String,
}

pub async fn get_profile(client: &reqwest::Client, token: &str) -> Result<APIProfile, Error> {
    let res = client
        .get("https://api.soundcloud.com/me")
        .header("Authorization", format!("OAuth {}", token))
        .header("accept", "application/json; charset=utf-8")
        .send()
        .await?
        .json::<APIProfile>()
        .await?;    
        Ok(res)
}