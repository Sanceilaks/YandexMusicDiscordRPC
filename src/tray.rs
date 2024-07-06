use std::process::exit;

use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
use winit::{
    event_loop, platform::windows::EventLoopBuilderExtWindows,
};

fn get_icon() -> Icon {
    let bytes = include_bytes!("icon.png");
    let image = image::load_from_memory(bytes).unwrap();
    let rgba = image.as_rgba8().unwrap();
    Icon::from_rgba(rgba.to_vec(), image.width(), image.height()).unwrap()
}

fn create_tray_menu() -> tray_icon::menu::Menu {
    let menu = tray_icon::menu::Menu::new();
    menu.append(
        &tray_icon::menu::MenuItemBuilder::new()
            .id("quit".into())
            .enabled(true)
            .text("Quit")
            .build(),
    )
    .unwrap();

    menu
}

pub fn start() -> tokio::task::JoinHandle<()> {
    let thread = tokio::spawn(async move {
        let mut icon: Option<TrayIcon> = None;

        let event_loop = event_loop::EventLoop::builder().with_any_thread(true).build().unwrap();
        let menu_channel = tray_icon::menu::MenuEvent::receiver();

        #[allow(deprecated)]
        event_loop.run(move |event, event_loop| {
            event_loop.set_control_flow(event_loop::ControlFlow::WaitUntil(
                std::time::Instant::now() + std::time::Duration::from_millis(16),
            ));

            #[cfg(not(target_os = "linux"))]
            if let winit::event::Event::NewEvents(winit::event::StartCause::Init) = event {
                icon = Some(
                    TrayIconBuilder::new()
                        .with_menu(Box::new(create_tray_menu().clone()))
                        .with_title("YandexMusicDiscordRPC")
                        .with_icon(get_icon())
                        .build()
                        .unwrap()
                )
            }

            if let Ok(event) = menu_channel.try_recv() {
                if event.id == "quit" {
                    icon.take();
                    exit(0);
                }
            }
        }).unwrap();
    });

    thread
}
