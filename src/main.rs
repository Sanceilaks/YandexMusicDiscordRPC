#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, sync::Arc};

use gsmtc::{ManagerEvent::*, SessionModel, SessionUpdateEvent::*};
use rpc::{State, YandexMusicState, YandexMusicStateBuilder};
use tokio::sync::Mutex;

mod rpc;
mod track_cache;
mod tray;
mod ymapi;

const TARGET_SOURCES: &[&str] = &["ru.yandex.desktop.music", "Yandex.Music"];

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    #[cfg(not(debug_assertions))]
    {
        let args: Vec<String> = env::args().collect();
        if args.contains(&"--debug".to_string()) {
            unsafe {
                use winapi::um::consoleapi::AllocConsole;
                AllocConsole();
            };
        }
    }

    env::set_var("RUST_LOG", "trace");

    env_logger::init();

    let track_cache = Arc::new(Mutex::new(track_cache::TrackCache::new()));

    let state_sender = Arc::new(rpc::init().await);

    let task = tray::start();

    let mut rx = gsmtc::SessionManager::create().await.unwrap();

	let mut sessions = Vec::new();

    while let Some(evt) = rx.recv().await {
		if let SessionRemoved { session_id } = evt {
			sessions.retain(|&x| x != session_id);
			trace!("Removed session: {session_id} {sessions:?}");

			if sessions.is_empty() {
				state_sender.send(rpc::RpcEvent::Clear).await.unwrap();
			}
		}
        if let SessionCreated {
            session_id,
            mut rx,
            source,
        } = evt
        {
            trace!("Created session: {{id={session_id}, source={source}}}");
            let state_sender = state_sender.clone();
            let track_cache = track_cache.clone();

			if TARGET_SOURCES.iter().any(|x| source.contains(x)) {
				sessions.push(session_id);
			}

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
                                trace!("Media update: {:?}", source);
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
                                let mut track_id: Option<usize> = None;
                                let mut album_id: Option<usize> = None;

                                if let Ok(track) = track_cache
                                    .lock()
                                    .await
                                    .get(media.title.clone(), media.artist.clone())
                                    .await
                                {
                                    img = track.as_ref().map(|x| x.thumbnail_uri.to_string());
                                    track_id = track.as_ref().map(|x| x.track_id);
                                    album_id = track.as_ref().map(|x| x.album_id);
                                }

                                current_state = Some(
                                    YandexMusicStateBuilder::new()
                                        .state(playback.unwrap().status.into())
                                        .album(
                                            media.album.map(|x| x.title).unwrap_or("".to_string()),
                                            album_id.unwrap_or(0),
                                        )
                                        .artist(media.artist)
                                        .track(media.title, track_id.unwrap_or(0))
                                        .image_url(
                                            img.as_ref().unwrap_or(&"logo".to_string()).clone(),
                                        )
                                        .build(),
                                );

                                state_sender
                                    .send(rpc::RpcEvent::Update(current_state.clone().unwrap()))
                                    .await
                                    .unwrap();
                            }
                        }
                        Model(model) => {
                            trace!("Model update: {:?}", source);

                            let SessionModel {
                                playback,
                                timeline: _,
                                media: _,
                                source,
                            } = model;

                            if playback.is_some()
                                && TARGET_SOURCES.iter().any(|x| source.contains(x))
                            {
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

                                        state_sender.send(rpc::RpcEvent::Update(state.clone())).await.unwrap();
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
