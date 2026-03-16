


pub async fn stream_track(client: &reqwest::Client, token: &str, id: u64) -> Result<Vec<u8>, reqwest::Error> {
       let res = client
        .get(format!("https://api.soundcloud.com/tracks/{}/stream", id))
        .header("Authorization", format!("OAuth {}", token))
        .header("accept", "application/json; charset=utf-8")
        .send()
        .await?
        .bytes()
        .await?
        .to_vec();
        Ok(res)
}