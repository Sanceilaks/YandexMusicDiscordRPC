use std::sync::LazyLock;

use discord_rich_presence::{
    activity::{Activity, Assets},
    DiscordIpc, DiscordIpcClient,
};

use dotenv_codegen::dotenv;
use gsmtc::PlaybackStatus;
use tokio::sync::Mutex;

#[derive(Clone, Copy)]
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
    pub artist: String,
    pub album: String,
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
                artist: String::new(),
                album: String::new(),
                state: State::Paused,
                image_url: String::new(),
            },
        }
    }

    pub fn track(mut self, track: String) -> Self {
        self.state.track = track;
        self
    }

    pub fn artist(mut self, artist: String) -> Self {
        self.state.artist = artist;
        self
    }

    pub fn album(mut self, album: String) -> Self {
        self.state.album = album;
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

pub struct RPC {
    client: DiscordIpcClient,
}

impl RPC {
    pub fn set_state(&mut self, state: YandexMusicState) {
        let YandexMusicState {
            track,
            artist,
            album,
            state,
            image_url,
        } = state;

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
                                if album.is_empty() { track.clone() } else { album }
                            ))
                            .small_image(match state {
                                State::Playing => "playing",
                                State::Paused => "paused",
                            })
                            .small_text(match state {
                                State::Playing => "▶️ Playing",
                                State::Paused => "⏸️ Paused",
                            }),
                    ),
            )
            .unwrap();
    }
}

static LAST_STATE: LazyLock<Mutex<Option<YandexMusicState>>> = LazyLock::new(|| Mutex::new(None));

#[allow(dead_code)]
pub async fn get_last_state() -> Option<YandexMusicState> {
    LAST_STATE.lock().await.clone()
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
            *LAST_STATE.lock().await = Some(evt.clone());
            rpc.set_state(evt);
        }

        rpc.client.close().unwrap();

        info!("Disconnected");
    });

    tx
}
