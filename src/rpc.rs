use discord_rich_presence::{
    activity::{Activity, Assets, Button},
    DiscordIpc, DiscordIpcClient,
};

use dotenv_codegen::dotenv;
use gsmtc::PlaybackStatus;
use reqwest::Url;

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Playing,
    Paused,
}

impl From<PlaybackStatus> for State {
    fn from(val: PlaybackStatus) -> Self {
        match val {
            PlaybackStatus::Playing => State::Playing,
            _ => State::Paused,
        }
    }
}

#[derive(Clone)]
pub struct YandexMusicState {
    pub track: String,
    pub track_id: usize,
    pub artist: String,
    pub album: String,
    pub album_id: usize,
    pub state: State,
    pub image_url: String,
}

pub struct YandexMusicStateBuilder {
    state: YandexMusicState,
}

impl YandexMusicStateBuilder {
    pub fn new() -> Self {
        Self {
            state: YandexMusicState {
                track: String::new(),
                track_id: 0,
                artist: String::new(),
                album: String::new(),
                album_id: 0,
                state: State::Paused,
                image_url: String::new(),
            },
        }
    }

    pub fn track(mut self, track: String, track_id: usize) -> Self {
        self.state.track = track;
        self.state.track_id = track_id;
        self
    }

    pub fn artist(mut self, artist: String) -> Self {
        self.state.artist = artist;
        self
    }

    pub fn album(mut self, album: String, album_id: usize) -> Self {
        self.state.album = album;
        self.state.album_id = album_id;
        self
    }

    pub fn state(mut self, state: State) -> Self {
        self.state.state = state;
        self
    }

    pub fn image_url(mut self, image_url: String) -> Self {
        self.state.image_url = image_url;
        self
    }

    pub fn build(self) -> YandexMusicState {
        self.state
    }
}

#[allow(clippy::upper_case_acronyms)]
pub struct RPC {
    client: DiscordIpcClient,
}

impl RPC {
    pub fn set_state(&mut self, state: YandexMusicState) {
        let YandexMusicState {
            track,
            track_id,
            artist,
            album,
            album_id,
            state,
            image_url,
        } = state;

        let track_url = Url::parse(&format!(
            "https://music.yandex/album/{album_id}/track/{track_id}"
        ))
        .unwrap();
        let track_search_url = Url::parse(&format!(
            "https://music.yandex/search?text={artist} - {track}"
        ))
        .unwrap();

        self.client
            .set_activity(
                Activity::new()
                    .details(&format!("🎵 {track}"))
                    .state(&format!("👤 {artist}"))
                    .assets(
                        Assets::new()
                            .large_image(&image_url)
                            .large_text(&format!(
                                "💿 {}",
                                if album.is_empty() {
                                    track.clone()
                                } else {
                                    album
                                }
                            ))
                            .small_image(match state {
                                State::Playing => "playing",
                                State::Paused => "paused",
                            })
                            .small_text(match state {
                                State::Playing => "▶️ Playing",
                                State::Paused => "⏸️ Paused",
                            }),
                    )
                    .buttons(vec![if track_id != 0 && album_id != 0 {
                        Button::new("🔗 Open URL", track_url.as_str())
                    } else {
                        Button::new("🔎 Open search", track_search_url.as_str())
                    }]),
            )
            .unwrap();
    }
}

pub async fn init() -> tokio::sync::mpsc::Sender<YandexMusicState> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<YandexMusicState>(10);

    tokio::spawn(async move {
        let mut rpc = RPC {
            client: DiscordIpcClient::new(dotenv!("CLIENT_ID")).unwrap(),
        };

        info!("Client ID: {}", dotenv!("CLIENT_ID"));

        while let Err(err) = rpc.client.connect().map_err(|x| x.to_string()) {
            error!("Failed to connect: {err}");
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }

        info!("Connected");

        while let Some(evt) = rx.recv().await {
            rpc.set_state(evt);
        }

        rpc.client.close().unwrap();

        info!("Disconnected");
    });

    tx
}
