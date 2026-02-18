use reqwest::{Error, Url};

#[derive(serde::Deserialize, Debug)]
pub struct APIProfile {
    pub avatar_url: Url,
    pub username: String,
    pub description: String,
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