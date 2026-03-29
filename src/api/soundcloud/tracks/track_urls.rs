use serde::{Deserialize};


#[derive(Deserialize)]
pub struct TrackUrls {
        pub _http_mp3_128_url: String,
        pub hls_mp3_128_url: String,
        pub _hls_aac_160_url: String,
        pub _preview_mp3_128_url: String,
}

pub async fn get_streaming_track_urls(client: &reqwest::Client, token: &str, urn: String) -> Result<TrackUrls, reqwest::Error> {
       let res = client
        .get(format!("https://api.soundcloud.com/tracks/{}/streams", urn))
        .header("Authorization", format!("OAuth {}", token))
        .header("accept", "application/json; charset=utf-8")
        .send()
        .await?
        .json::<TrackUrls>()
        .await?;
        Ok(res)
}