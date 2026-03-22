
#[derive(serde::Deserialize)]
pub struct APIPlaylistTracks {
        pub id: u64,
        pub urn: String,
        pub streamable: bool,
        pub title: String,
        pub description: Option<String>,
}

pub async fn get_playlist_tracks(client: &reqwest::Client, token: &str, playlist_urn: String) -> Result<Vec<APIPlaylistTracks>, reqwest::Error> {
   let res = client
        .get(format!("https://api.soundcloud.com/playlists/{}/tracks", playlist_urn))
        .header("Authorization", format!("OAuth {}", token))
        .header("accept", "application/json; charset=utf-8")
        .send()
        .await?
        .json::<Vec<APIPlaylistTracks>>()
        .await?;
        Ok(res)
}