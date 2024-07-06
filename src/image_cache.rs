use std::collections::HashMap;

use reqwest::Url;

use crate::ymapi;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct Request {
    title: String,
    artist: String,
}

pub struct ImageCache {
    cache: HashMap<Request, Url>,
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub async fn get(&mut self, title: String, artist: String) -> Result<Option<Url>, Box<dyn std::error::Error>> {
        let request = Request { title: title.clone(), artist: artist.clone() };

        if let Some(url) = self.cache.get(&request) {
            return Ok(Some(url.clone()));
        }

        let result = ymapi::search(&title, &artist).await?;

        if let Some(track) = result {
            let url = track.get_thumbnail();
            let url = Url::parse(&url)?;

            self.cache.insert(request, url.clone());
            Ok(Some(url))
        } else {
            Ok(None)
        }
    }
}
