use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

const BASE_URL: &str = " https://api.music.yandex.net/search";

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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Tracks {
    results: Vec<Track>,
    total: usize,

    #[serde(rename = "perPage")]
    per_page: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResults {
    text: String,
    tracks: Tracks,
    page: usize,

    #[serde(rename = "type")]
    _type: String,
}

impl SearchResults {
    pub async fn next_page(
        &self,
        client: &Client,
    ) -> Result<SearchResults, Box<dyn std::error::Error>> {
        let url = Url::parse_with_params(
            BASE_URL,
            [
                ("page", &(self.page + 1).to_string()),
                ("text", &self.text),
                ("type", &self._type),
            ],
        )
        .unwrap();

        trace!("Fetching next page: {url}");
        let response = client
            .get(url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(serde_json::from_value::<SearchResults>(
            response
                .get("result")
                .unwrap_or(&serde_json::Value::Null)
                .to_owned(),
        )?)
    }
}

pub async fn create_client() -> Result<Client, reqwest::Error> {
    static CLIENT: OnceCell<Client> = OnceCell::const_new();

    CLIENT
        .get_or_try_init(|| async {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "X-Yandex-Music-Client",
                "YandexMusicAndroid/24023621".parse().unwrap(),
            );

            Client::builder()
                .user_agent("Yandex-Music-API")
                .default_headers(headers)
                .build()
        })
        .await
        .cloned()
}

pub async fn search(
    title: &String,
    artist: &str,
) -> Result<Option<Track>, Box<dyn std::error::Error>> {
    let url = Url::parse_with_params(
        BASE_URL,
        [
            ("page", &"0".to_string()),
            ("text", title),
            ("type", &"track".to_string()),
        ],
    )?;

    trace!("Searching: {url}");

    let client = create_client().await?;
    let response = client
        .get(url.clone())
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let first_page: SearchResults = serde_json::from_value(
        response
            .get("result")
            .unwrap_or(&serde_json::Value::Null)
            .to_owned(),
    )?;

    for track in &first_page.tracks.results {
        if track.artists.iter().all(|x| artist.contains(&x.name)) {
            return Ok(Some(track.clone()));
        }
    }

    trace!("Searching next page: {url}");

    let mut results: SearchResults = first_page.clone();

    trace!("Found: {}", results.tracks.results.len());

    for _ in 0..((first_page.tracks.total / first_page.tracks.per_page) - 1) {
        results = results.next_page(&client).await?;

        for track in &results.tracks.results {
            if track.artists.iter().all(|x| artist.contains(&x.name)) {
                return Ok(Some(track.clone()));
            }
        }
    }

    Ok(None)
}
