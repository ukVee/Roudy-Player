

pub async fn track_metadata(client: &reqwest::Client, token: &str, id: u64) -> Result<String, reqwest::Error> {
       let res = client
        .get(format!("https://api.soundcloud.com/tracks/{}", id))
        .header("Authorization", format!("OAuth {}", token))
        .header("accept", "application/json; charset=utf-8")
        .send()
        .await?
        .text()
        .await?;
        Ok(res)
}