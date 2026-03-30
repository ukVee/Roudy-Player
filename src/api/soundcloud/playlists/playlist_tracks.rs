
#[derive(serde::Deserialize, Clone)]
pub struct APIPlaylistTracks {
        pub id: u64,
        pub urn: String,
        pub streamable: bool,
        pub title: String,
        pub description: Option<String>,
        pub duration: u64,
        pub tag_list: String,
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