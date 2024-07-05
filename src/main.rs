use std::sync::Arc;

use gsmtc::{ManagerEvent::*, SessionModel, SessionUpdateEvent::*};
use rpc::YandexMusicStateBuilder;

mod rpc;
mod tray;

//TODO: Old client
const TARGET_SOURCES: &[&str] = &["ru.yandex.desktop.music"];

#[tokio::main]
async fn main() {
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
            println!("Created session: {{id={session_id}, source={source}}}");
            let state_sender = state_sender.clone();
            tokio::spawn(async move {
                let mut img: Option<String> = None;

                while let Some(evt) = rx.recv().await {
                    match evt {
                        Media(model, image) => {
                            if img.is_none() {
                                if let Some(image) = image {
                                    let client = reqwest::Client::new();
                                    let res = client
                                        .post("https://paste.rs")
                                        .body(image.data)
                                        .send()
                                        .await
                                        .unwrap();

                                    img = Some(res.text().await.unwrap());
                                }
                            }

                            let SessionModel {
                                playback,
                                timeline: _,
                                media,
                                source,
                            } = model;

                            if TARGET_SOURCES.contains(&source.as_str()) {
                                let media = media.unwrap();

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
                                                img.as_ref().unwrap_or(&"logo".to_string()).clone(),
                                            )
                                            .build(),
                                    )
                                    .await
                                    .unwrap();
                            }
                        }
                        Model(model) => {
                            let SessionModel {
                                playback,
                                timeline: _,
                                media,
                                source,
                            } = model;

                            if playback.is_some() && media.is_some() && TARGET_SOURCES.contains(&source.as_str()) {
                                let media = media.unwrap();

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
                                            .image_url(
                                                img.as_ref()
                                                    .unwrap_or(&"logo".to_string())
                                                    .clone(),
                                            )
                                            .track(media.title)
                                            .build(),
                                    )
                                    .await
                                    .unwrap();
                            }
                        }
                    }
                }
            });
        }
    }

    task.await.unwrap();
}
