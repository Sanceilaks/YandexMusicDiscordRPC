#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{env, sync::Arc};

use gsmtc::{ManagerEvent::*, SessionModel, SessionUpdateEvent::*};
use rpc::YandexMusicStateBuilder;

mod rpc;
mod tray;
mod ymapi;

const TARGET_SOURCES: &[&str] = &["ru.yandex.desktop.music", "Yandex.Music"];

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");

    #[cfg(debug_assertions)]
    env::set_var("RUST_LOG", "trace");

    env_logger::init();

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
            tokio::spawn(async move {
                let mut current_song: Option<(String, String)> = None;

                while let Some(evt) = rx.recv().await {
                    if let Media(model, _) = evt {
                        let SessionModel {
                            playback,
                            timeline: _,
                            media,
                            source,
                        } = model;

                        if TARGET_SOURCES.iter().any(|x| source.contains(x)) {
                            let media = media.unwrap();
                            if media.title.is_empty() || media.artist.is_empty() {
                                continue;
                            }

                            if let Some((track, artist)) = current_song.to_owned() {
                                if media.title.eq(&track) && media.artist.eq(&artist) {
                                    continue;
                                }
                            }

                            current_song = Some((media.title.clone(), media.artist.clone()));

                            if rpc::get_last_state()
                                .await
                                .map(|x| {
                                    x.track.eq(&media.title)
                                })
                                .unwrap_or(false)
                            {
                                continue;
                            }

                            let mut img: Option<String> = None;

                            trace!("Media update: {:?}", source);

                            let track =
                                ymapi::search(&media.title, &media.artist).await.unwrap();
                            if let Some(track) = track {
                                img = Some(track.get_thumbnail());
                                trace!("Got image: {}", img.as_ref().unwrap());
                            }

                            state_sender
                                .send(
                                    YandexMusicStateBuilder::new()
                                        .state(playback.unwrap().status.into())
                                        .album(
                                            media
                                                .album
                                                .map(|x| x.title)
                                                .unwrap_or("".to_string()),
                                        )
                                        .artist(media.artist)
                                        .track(media.title)
                                        .image_url(
                                            img.as_ref()
                                                .unwrap_or(&"https://pic.re/image".to_string())
                                                .clone(),
                                        )
                                        .build(),
                                )
                                .await
                                .unwrap();
                        }
                    }
                }
            });
        }
    }

    task.await.unwrap();
}
