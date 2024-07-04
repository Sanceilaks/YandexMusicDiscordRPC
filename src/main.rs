use gsmtc::{ManagerEvent::*, SessionUpdateEvent::*};

mod tray;

#[tokio::main]
async fn main() {
    let task = tray::start();

    let mut rx = gsmtc::SessionManager::create().await.unwrap();

    while let Some(evt) = rx.recv().await {
        if let SessionCreated { session_id, mut rx, source } = evt {
            println!("Created session: {{id={session_id}, source={source}}}");
            tokio::spawn(async move {
                while let Some(evt) = rx.recv().await {
                    if let Media(model, image) = evt {
                        println!("[{session_id}/{source}] Media updated: {model:#?} - {image:?}");
                    }
                }
            });
        }
    }

    task.await.unwrap();
}
