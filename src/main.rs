use gsmtc::{ManagerEvent::*, SessionUpdateEvent::*};

mod tray;

#[tokio::main]
async fn main() {
    let task = tray::start();

    let mut rx = gsmtc::SessionManager::create().await.unwrap();

    while let Some(evt) = rx.recv().await {
        match evt {
            SessionCreated { session_id, mut rx, source } => {
                println!("Created session: {{id={session_id}, source={source}}}");
                tokio::spawn(async move {
                    while let Some(evt) = rx.recv().await {
                        match evt {
                            Media(model, image) => {
                                println!("[{session_id}/{source}] Media updated: {model:#?} - {image:?}");
                            },
                            _ => {}
                        }
                    }
                });
            },
            _ => {}
        }
    }

    task.await.unwrap();
}
