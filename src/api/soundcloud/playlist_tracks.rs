


pub async fn get_playlist_tracks(client: &reqwest::Client, token: &str, playlist_urn: String) -> Result<String, reqwest::Error> {
   let res = client
        .get(format!("https://api.soundcloud.com/playlists/{}/tracks", playlist_urn))
        .header("Authorization", format!("OAuth {}", token))
        .header("accept", "application/json; charset=utf-8")
        .send()
        .await?
        .text()
        .await?;
        Ok(res)
}