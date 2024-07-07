use std::collections::HashMap;

use reqwest::Url;

use crate::ymapi;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct Request {
    title: String,
    artist: String,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct CacheEntity {
    pub thumbnail_uri: String,
    pub album_id: usize,
    pub track_id: usize,
}

pub struct TrackCache {
    cache: HashMap<Request, Option<CacheEntity>>,
}

impl TrackCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub async fn get(
        &mut self,
        title: String,
        artist: String,
    ) -> Result<Option<CacheEntity>, Box<dyn std::error::Error>> {
        let request = Request {
            title: title.clone(),
            artist: artist.clone(),
        };

        if let Some(entity) = self.cache.get(&request) {
            return Ok(entity.clone());
        }

        let result = ymapi::search(&title, &artist).await.inspect_err(|err| {
            error!("Failed to search: {err}");
        });

        let result = result.unwrap_or_default();

        trace!("Search result: {result:?}");

        if let Some(track) = result {
            let url = track.get_thumbnail();
            let url = Url::parse(&url)?;

            let entity = CacheEntity {
                thumbnail_uri: url.to_string(),
                album_id: track.albums.get(0).map(|x| x.id).unwrap_or(0),
                track_id: track.id,
            };

            self.cache.insert(request, Some(entity.clone()));

            Ok(Some(entity))
        } else {
            self.cache.insert(request, None);
            Ok(None)
        }
    }
}
