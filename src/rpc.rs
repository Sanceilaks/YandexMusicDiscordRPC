
use discord_rich_presence::{activity::{Activity, Assets}, DiscordIpc, DiscordIpcClient};

use dotenv_codegen::dotenv;
use gsmtc::PlaybackStatus;

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
        } = state;

        self.client
            .set_activity( Activity::new()
                .details(&format!("🎵 {track}"))
                    .state(&format!("👤 {artist}"))
                    .assets(
                        Assets::new()
                            .large_image("logo")
                            .large_text(&format!("💿 {album}"))
                            .small_image(match state {
                                State::Playing => "playing",
                                State::Paused => "paused",
                            })
                            .small_text(match state {
                                State::Playing => "▶️ Playing",
                                State::Paused => "⏸️ Paused",
                            })
                    )
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

        println!("Client ID: {}", dotenv!("CLIENT_ID"));

        rpc.client.connect().unwrap();

        println!("Connected");

        while let Some(evt) = rx.recv().await {
            rpc.set_state(evt);
        }
    });

    tx
}