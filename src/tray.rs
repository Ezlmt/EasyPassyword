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
    tray_icon::Icon::from_rgba(generate_tray_icon_rgba_32(), 32, 32).expect("invalid tray icon")
}

fn generate_tray_icon_rgba_32() -> Vec<u8> {
    const W: usize = 32;
    const H: usize = 32;

    // Background + bars palette.
    let bg = [0x15u8, 0x15u8, 0x15u8]; // #151515
    let bar = [0x00u8, 0xE6u8, 0x76u8]; // #00E676

    // Signed distance to a rounded rectangle centered at (cx, cy).
    fn sd_round_rect(px: f32, py: f32, cx: f32, cy: f32, hw: f32, hh: f32, r: f32) -> f32 {
        let dx = (px - cx).abs() - hw;
        let dy = (py - cy).abs() - hh;
        let ax = dx.max(0.0);
        let ay = dy.max(0.0);
        let outside = (ax * ax + ay * ay).sqrt();
        let inside = dx.max(dy).min(0.0);
        outside + inside - r
    }

    fn smooth_alpha(d: f32) -> f32 {
        // ~1px soft edge.
        let a = 0.5 - d;
        a.clamp(0.0, 1.0)
    }

    let mut out = vec![0u8; W * H * 4];

    // Layout in pixel space.
    let cx = 16.0;
    let cy = 16.0;

    // Outer squircle.
    let outer_hw = 10.0;
    let outer_hh = 10.0;
    let outer_r = 7.0;

    // Two vertical pills hinting at ';;'.
    let pill_w = 3.0;
    let pill_h = 7.0;
    let pill_r = 3.0;
    let left_x = 12.0;
    let right_x = 20.0;

    for y in 0..H {
        for x in 0..W {
            // 2x2 supersampling for crisper edges.
            let mut a_outer = 0.0;
            let mut a_left = 0.0;
            let mut a_right = 0.0;
            for sy in 0..2 {
                for sx in 0..2 {
                    let px = x as f32 + (sx as f32 + 0.5) * 0.5;
                    let py = y as f32 + (sy as f32 + 0.5) * 0.5;

                    let d_outer = sd_round_rect(px, py, cx, cy, outer_hw, outer_hh, outer_r);
                    a_outer += smooth_alpha(d_outer);

                    let d_left = sd_round_rect(px, py, left_x, cy, pill_w, pill_h, pill_r);
                    a_left += smooth_alpha(d_left);

                    let d_right = sd_round_rect(px, py, right_x, cy, pill_w, pill_h, pill_r);
                    a_right += smooth_alpha(d_right);
                }
            }
            a_outer *= 0.25;
            a_left *= 0.25;
            a_right *= 0.25;

            // Bars only render inside the outer shape.
            let a_bar = (a_left.max(a_right)).min(a_outer);

            // Composite: outer bg + bars over it.
            let mut r = 0.0;
            let mut g = 0.0;
            let mut b = 0.0;
            if a_outer > 0.0 {
                r = bg[0] as f32 / 255.0;
                g = bg[1] as f32 / 255.0;
                b = bg[2] as f32 / 255.0;
            }

            if a_bar > 0.0 {
                let br = bar[0] as f32 / 255.0;
                let bgc = bar[1] as f32 / 255.0;
                let bb = bar[2] as f32 / 255.0;
                // Over operator with a_bar over a_outer.
                r = r * (1.0 - a_bar) + br * a_bar;
                g = g * (1.0 - a_bar) + bgc * a_bar;
                b = b * (1.0 - a_bar) + bb * a_bar;
            }

            let a = a_outer.max(a_bar);

            let i = (y * W + x) * 4;
            out[i] = (r * 255.0).round() as u8;
            out[i + 1] = (g * 255.0).round() as u8;
            out[i + 2] = (b * 255.0).round() as u8;
            out[i + 3] = (a * 255.0).round() as u8;
        }
    }

    out
}
