

pub async fn get_track_manifest(client: &reqwest::Client, token: &str, url: String) -> Result<String, reqwest::Error> {
       let res = client
        .get(format!("{}", url))
        .header("Authorization", format!("OAuth {}", token))
        .header("accept", "application/json; charset=utf-8")
        .send()
        .await?
        .text()
        .await?;
        
        
        Ok(res)
}