use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Artist {
    id: usize,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Track {
    id: usize,
    title: String,
    artists: Vec<Artist>,

    #[serde(rename = "coverUri")]
    cover_uri: String,
}

impl Track {
    pub fn get_thumbnail(&self) -> String {
        trace!("Getting thumbnail: {}", self.cover_uri);
        format!("https://{}", self.cover_uri.replace("%%", "200x200"))
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Tracks {
    items: Vec<Track>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResults {
    text: String,
    tracks: Tracks,
}

pub async fn search(title: &String, artist: &String) -> Result<Option<Track>, Box<dyn std::error::Error>> {
    static BASE_URL: &str = "https://music.yandex.ru/handlers/music-search.jsx";
    let url = Url::parse_with_params(BASE_URL, [("text", title), ("type", &"tracks".to_string())])?;

    trace!("Searching: {url}");

    let response = reqwest::get(url).await?;
    let results: SearchResults = response.json().await?;

    trace!("Track count: {}", results.tracks.items.len());

    for track in results.tracks.items {
        if track.artists.iter().all(|x| artist.contains(&x.name)) {
            return Ok(Some(track));
        }
    }

    Ok(None)
}