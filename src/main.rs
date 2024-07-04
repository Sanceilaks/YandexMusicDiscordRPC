
use std::sync::Arc;

use gsmtc::{
    ManagerEvent::*,
    SessionModel,
    SessionUpdateEvent::{*},
};
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
                while let Some(evt) = rx.recv().await {
                    if let Media(model, image) = evt {
                        println!("[{session_id}/{source}] Media updated: {model:#?} - {image:?}");

                        let SessionModel {
                            playback,
                            timeline: _,
                            media,
                            source,
                        } = model;

                        if TARGET_SOURCES.contains(&source.as_str()) {
                            let media = media.unwrap();

                            state_sender.send(
                                YandexMusicStateBuilder::new()
                                    .state(playback.unwrap().status.into())
                                    .album(media.album.map(|x| x.title).unwrap_or("".to_string()))
                                    .artist(media.artist)
                                    .track(media.title)
                                    .build()
                            ).await.unwrap();
                        }
                    }
                }
            });
        }
    }

    task.await.unwrap();
}
