use crossbeam_channel::Sender;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, EventField,
};

use crate::core::GenerationMode;
use crate::error::Result;

use super::{TriggerDetector, TriggerEvent};

fn keycode_to_key(keycode: u16) -> Option<rdev::Key> {
    match keycode {
        0 => Some(rdev::Key::KeyA),
        1 => Some(rdev::Key::KeyS),
        2 => Some(rdev::Key::KeyD),
        3 => Some(rdev::Key::KeyF),
        4 => Some(rdev::Key::KeyH),
        5 => Some(rdev::Key::KeyG),
        6 => Some(rdev::Key::KeyZ),
        7 => Some(rdev::Key::KeyX),
        8 => Some(rdev::Key::KeyC),
        9 => Some(rdev::Key::KeyV),
        11 => Some(rdev::Key::KeyB),
        12 => Some(rdev::Key::KeyQ),
        13 => Some(rdev::Key::KeyW),
        14 => Some(rdev::Key::KeyE),
        15 => Some(rdev::Key::KeyR),
        16 => Some(rdev::Key::KeyY),
        17 => Some(rdev::Key::KeyT),
        18 => Some(rdev::Key::Num1),
        19 => Some(rdev::Key::Num2),
        20 => Some(rdev::Key::Num3),
        21 => Some(rdev::Key::Num4),
        22 => Some(rdev::Key::Num6),
        23 => Some(rdev::Key::Num5),
        24 => Some(rdev::Key::Equal),
        25 => Some(rdev::Key::Num9),
        26 => Some(rdev::Key::Num7),
        27 => Some(rdev::Key::Minus),
        28 => Some(rdev::Key::Num8),
        29 => Some(rdev::Key::Num0),
        30 => Some(rdev::Key::RightBracket),
        31 => Some(rdev::Key::KeyO),
        32 => Some(rdev::Key::KeyU),
        33 => Some(rdev::Key::LeftBracket),
        34 => Some(rdev::Key::KeyI),
        35 => Some(rdev::Key::KeyP),
        36 => Some(rdev::Key::Return),
        37 => Some(rdev::Key::KeyL),
        38 => Some(rdev::Key::KeyJ),
        39 => Some(rdev::Key::Quote),
        40 => Some(rdev::Key::KeyK),
        41 => Some(rdev::Key::SemiColon),
        42 => Some(rdev::Key::BackSlash),
        43 => Some(rdev::Key::Comma),
        44 => Some(rdev::Key::Slash),
        45 => Some(rdev::Key::KeyN),
        46 => Some(rdev::Key::KeyM),
        47 => Some(rdev::Key::Dot),
        48 => Some(rdev::Key::Tab),
        49 => Some(rdev::Key::Space),
        50 => Some(rdev::Key::BackQuote),
        51 => Some(rdev::Key::Backspace),
        _ => None,
    }
}

pub fn start_keyboard_listener_macos(
    tx: Sender<TriggerEvent>,
    triggers: Vec<(String, GenerationMode)>,
    injection_active: Arc<AtomicBool>,
) -> Result<thread::JoinHandle<()>> {
    let handle = thread::spawn(move || {
        let detector = RefCell::new(TriggerDetector::new(
            triggers.clone(),
            injection_active.clone(),
        ));

        log::info!(
            "[LISTENER-MACOS] Keyboard listener started, triggers: {:?}",
            triggers
        );

        let tx_clone = tx.clone();
        let injection_clone = injection_active.clone();

        let tap = CGEventTap::new(
            CGEventTapLocation::Session,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![CGEventType::KeyDown],
            move |_proxy, _event_type, event: &CGEvent| {
                if injection_clone.load(Ordering::SeqCst) {
                    return None;
                }

                let keycode =
                    event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
                let flags = event.get_flags();

                if flags.contains(CGEventFlags::CGEventFlagCommand)
                    || flags.contains(CGEventFlags::CGEventFlagControl)
                    || flags.contains(CGEventFlags::CGEventFlagAlternate)
                {
                    return None;
                }

                if let Some(key) = keycode_to_key(keycode) {
                    log::info!("[MACOS-KEY] keycode={} -> {:?}", keycode, key);

                    let rdev_event = rdev::Event {
                        time: std::time::SystemTime::now(),
                        name: None,
                        event_type: rdev::EventType::KeyPress(key),
                    };

                    if let Some(trigger) = detector.borrow_mut().process_event(&rdev_event) {
                        log::info!("[SEND-MACOS] Sending trigger event: {:?}", trigger);
                        if let Err(e) = tx_clone.send(trigger) {
                            log::error!("[ERROR-MACOS] Failed to send trigger: {}", e);
                        }
                    }
                }

                None
            },
        );

        match tap {
            Ok(tap) => unsafe {
                let loop_source = tap
                    .mach_port
                    .create_runloop_source(0)
                    .expect("failed to create runloop source");
                let run_loop = CFRunLoop::get_current();
                run_loop.add_source(&loop_source, kCFRunLoopCommonModes);
                tap.enable();
                log::info!("[LISTENER-MACOS] CGEventTap enabled, starting run loop...");
                CFRunLoop::run_current();
            },
            Err(()) => {
                log::error!("[ERROR-MACOS] Failed to create CGEventTap. Make sure the app has Input Monitoring permissions in System Settings > Privacy & Security > Input Monitoring.");
            }
        }

        log::info!("[LISTENER-MACOS] Run loop exited");
    });

    Ok(handle)
}
