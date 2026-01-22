use crossbeam_channel::{Receiver, Sender};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};

use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIconBuilder, TrayIconEvent};

use crate::{ControlCommand, TrayUpdate};

fn set_control_flow(control_flow: &mut ControlFlow) {
    *control_flow = ControlFlow::Wait;
}

#[derive(Debug, Clone)]
enum UserEvent {
    TrayIconEvent(tray_icon::TrayIconEvent),
    MenuEvent(tray_icon::menu::MenuEvent),
}

fn forward_tray_event(
    proxy: EventLoopProxy<UserEvent>,
) -> impl Fn(tray_icon::TrayIconEvent) + Send + Sync + 'static {
    move |event| {
        let _ = proxy.send_event(UserEvent::TrayIconEvent(event));
    }
}

fn forward_menu_event(
    proxy: EventLoopProxy<UserEvent>,
) -> impl Fn(tray_icon::menu::MenuEvent) + Send + Sync + 'static {
    move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }
}

pub fn run_tray(
    command_tx: Sender<ControlCommand>,
    tray_update_rx: Receiver<TrayUpdate>,
    initial_autostart: bool,
) -> ! {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(forward_tray_event(proxy.clone())));
    MenuEvent::set_event_handler(Some(forward_menu_event(proxy)));

    let open_config = MenuItem::new("Open Config", true, None);
    let reload_config = MenuItem::new("Reload Config", true, None);
    let start_on_login = CheckMenuItem::new("Start on Login", true, initial_autostart, None);
    let quit = MenuItem::new("Exit", true, None);

    let tray_menu = Menu::new();
    tray_menu
        .append_items(&[
            &open_config,
            &reload_config,
            &start_on_login,
            &PredefinedMenuItem::separator(),
            &quit,
        ])
        .expect("failed to build tray menu");

    let mut tray_icon = None;
    let mut autostart_enabled = initial_autostart;
    let mut autostart_revert_to: Option<bool> = None;

    event_loop.run(move |event, _, control_flow| {
        set_control_flow(control_flow);

        match event {
            Event::NewEvents(StartCause::Init) => {
                let icon = default_icon();
                tray_icon = Some(
                    TrayIconBuilder::new()
                        .with_menu(Box::new(tray_menu.clone()))
                        .with_tooltip("EasyPassword")
                        .with_icon(icon)
                        .build()
                        .expect("failed to create tray icon"),
                );

                #[cfg(target_os = "macos")]
                unsafe {
                    use objc2_core_foundation::{CFRunLoopGetMain, CFRunLoopWakeUp};
                    let rl = CFRunLoopGetMain().unwrap();
                    CFRunLoopWakeUp(&rl);
                }
            }
            Event::UserEvent(UserEvent::MenuEvent(event)) => {
                if event.id == open_config.id() {
                    let _ = command_tx.send(ControlCommand::OpenConfig);
                } else if event.id == reload_config.id() {
                    let _ = command_tx.send(ControlCommand::ReloadConfig);
                } else if event.id == start_on_login.id() {
                    let requested = !autostart_enabled;
                    autostart_revert_to = Some(autostart_enabled);
                    autostart_enabled = requested;
                    start_on_login.set_checked(requested);
                    let _ = command_tx.send(ControlCommand::SetAutostart(requested));
                } else if event.id == quit.id() {
                    let _ = command_tx.send(ControlCommand::Exit);
                    tray_icon.take();
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::UserEvent(UserEvent::TrayIconEvent(_event)) => {}
            Event::MainEventsCleared => {
                while let Ok(update) = tray_update_rx.try_recv() {
                    match update {
                        TrayUpdate::AutostartSetResult { enabled, ok, error } => {
                            if ok {
                                autostart_revert_to = None;
                                autostart_enabled = enabled;
                                start_on_login.set_checked(enabled);
                            } else {
                                let fallback = autostart_revert_to.unwrap_or(enabled);
                                autostart_revert_to = None;
                                autostart_enabled = fallback;
                                start_on_login.set_checked(fallback);
                                if let Some(error) = error {
                                    log::error!("failed to set autostart: {}", error);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    });
}

fn default_icon() -> tray_icon::Icon {
    const RGBA: &[u8] = include_bytes!("../assets/icon-tray-32.rgba");
    tray_icon::Icon::from_rgba(RGBA.to_vec(), 32, 32).expect("invalid tray icon")
}
