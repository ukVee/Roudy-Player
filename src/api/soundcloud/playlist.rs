use reqwest::Error;


#[derive(serde::Deserialize)]
pub struct APIPlaylist {
    permalink: String,
    duration: i64,
    track_count: i32,
}

pub async fn get_playlists(client: &reqwest::Client, token: &str) -> Result<Vec<APIPlaylist>, Error> {
    let res = client
        .get("https://api.soundcloud.com/me/playlists")
        .header("Authorization", format!("OAuth {}", token))
        .header("accept", "application/json; charset=utf-8")
        .send()
        .await?
        .json::<Vec<APIPlaylist>>()
        .await?;
        Ok(res)
}