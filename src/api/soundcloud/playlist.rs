use reqwest::Error;



pub async fn get_playlists(client: &reqwest::Client, token: &str) -> Result<String, Error> {
    let res = client
        .get("https://api.soundcloud.com/playlists")
        .header("Authorization", format!("OAuth {}", token))
        .header("accept", "application/json; charset=utf-8")
        .send()
        .await?
        .text()
        .await?;
        Ok(res)
}