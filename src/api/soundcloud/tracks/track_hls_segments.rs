use bytes::Bytes;

pub async fn get_track_segments(client: &reqwest::Client, token: &str, url: String) -> Result<Bytes, reqwest::Error> {
       let res = client
        .get(format!("{}", url))
        .header("Authorization", format!("OAuth {}", token))
        .header("accept", "application/json; charset=utf-8")
        .send()
        .await?
        .bytes()
        .await?;
        Ok(res)
}