#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, sync::Arc};

use gsmtc::{ManagerEvent::*, SessionModel, SessionUpdateEvent::*};
use rpc::{State, YandexMusicState, YandexMusicStateBuilder};
use tokio::sync::Mutex;

mod image_cache;
mod rpc;
mod tray;
mod ymapi;

const TARGET_SOURCES: &[&str] = &["ru.yandex.desktop.music", "Yandex.Music"];

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    #[cfg(not(debug_assertions))]
    {
        use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS};
        unsafe {
            AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }


    env::set_var("RUST_LOG", "info");

    #[cfg(debug_assertions)]
    env::set_var("RUST_LOG", "trace");

    env_logger::init();

    let image_cache = Arc::new(Mutex::new(image_cache::ImageCache::new()));

    let state_sender = Arc::new(rpc::init().await);

    let task = tray::start();

    let mut rx = gsmtc::SessionManager::create().await.unwrap();

    while let Some(evt) = rx.recv().await {
        if let SessionCreated {
            session_id,
            mut rx,
            source,
        } = evt
        {
            trace!("Created session: {{id={session_id}, source={source}}}");
            let state_sender = state_sender.clone();
            let image_cache = image_cache.clone();
            tokio::spawn(async move {
                let mut current_state: Option<YandexMusicState> = None;

                while let Some(evt) = rx.recv().await {
                    match evt {
                        Media(model, _) => {
                            let SessionModel {
                                playback,
                                timeline: _,
                                media,
                                source,
                            } = model;

                            if TARGET_SOURCES.iter().any(|x| source.contains(x)) {
                                let media = media.unwrap();
                                let is_playing = playback
                                    .as_ref()
                                    .unwrap()
                                    .status
                                    .eq(&gsmtc::PlaybackStatus::Playing);

                                if media.title.is_empty() || media.artist.is_empty() {
                                    continue;
                                }

                                if let Some(state) = current_state.to_owned() {
                                    if media.title.eq(&state.track)
                                        && media.artist.eq(&state.artist)
                                        && is_playing == (state.state == State::Playing)
                                    {
                                        continue;
                                    }
                                }

                                let mut img: Option<String> = None;

                                trace!("Media update: {:?}", source);

                                if let Ok(image) = image_cache
                                    .lock()
                                    .await
                                    .get(media.title.clone(), media.artist.clone())
                                    .await
                                {
                                    img = image.map(|x| x.to_string());
                                }

                                current_state = Some(
                                    YandexMusicStateBuilder::new()
                                        .state(playback.unwrap().status.into())
                                        .album(
                                            media.album.map(|x| x.title).unwrap_or("".to_string()),
                                        )
                                        .artist(media.artist)
                                        .track(media.title)
                                        .image_url(
                                            img.as_ref()
                                                .unwrap_or(&"https://pic.re/image".to_string())
                                                .clone(),
                                        )
                                        .build(),
                                );

                                state_sender
                                    .send(current_state.clone().unwrap())
                                    .await
                                    .unwrap();
                            }
                        }
                        Model(model) => {
                            let SessionModel {
                                playback,
                                timeline: _,
                                media: _,
                                source,
                            } = model;

                            if playback.is_some() && TARGET_SOURCES.iter().any(|x| source.contains(x)) {
                                let is_playing = playback
                                    .as_ref()
                                    .unwrap()
                                    .status
                                    .eq(&gsmtc::PlaybackStatus::Playing);

                                if let Some(state) = current_state.as_mut() {
                                    if is_playing != (state.state == State::Playing) {
                                        state.state = if is_playing {
                                            State::Playing
                                        } else {
                                            State::Paused
                                        };

                                        state_sender.send(state.clone()).await.unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
    }

    task.await.unwrap();
}
