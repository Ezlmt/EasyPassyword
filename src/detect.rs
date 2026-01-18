use crossbeam_channel::Sender;
use rdev::{listen, Event, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub site: String,
    pub trigger_len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DetectorState {
    Idle,
    Prefix1,
    Prefix2,
    CollectingSite,
}

pub struct TriggerDetector {
    state: DetectorState,
    buffer: String,
    prefix: String,
    injection_active: Arc<AtomicBool>,
}

impl TriggerDetector {
    pub fn new(prefix: String, injection_active: Arc<AtomicBool>) -> Self {
        Self {
            state: DetectorState::Idle,
            buffer: String::new(),
            prefix,
            injection_active,
        }
    }

    pub fn process_event(&mut self, event: &Event) -> Option<TriggerEvent> {
        if self.injection_active.load(Ordering::SeqCst) {
            log::debug!("[SKIP] Injection active, ignoring event");
            return None;
        }

        match &event.event_type {
            EventType::KeyPress(key) => {
                log::info!(
                    "[KEY] Press: {:?} | name: {:?} | state: {:?} | buffer: \"{}\"",
                    key,
                    event.name,
                    self.state,
                    self.buffer
                );
                self.handle_key_press(*key, event)
            }
            EventType::KeyRelease(key) => {
                log::debug!("[KEY] Release: {:?}", key);
                None
            }
            _ => None,
        }
    }

    fn handle_key_press(&mut self, key: Key, event: &Event) -> Option<TriggerEvent> {
        if key == Key::Backspace {
            self.handle_backspace();
            log::info!("[BACKSPACE] buffer now: \"{}\"", self.buffer);
            return None;
        }

        if is_terminator(key) {
            log::info!("[TERMINATOR] {:?} pressed, checking trigger...", key);
            return self.handle_terminator();
        }

        let ch = match &event.name {
            Some(name) if !name.is_empty() => {
                let c = name.chars().next();
                log::debug!("[CHAR] from event.name: {:?}", c);
                c?
            }
            _ => {
                let c = key_to_char(key);
                log::debug!("[CHAR] from key_to_char: {:?}", c);
                c?
            }
        };

        self.process_char(ch)
    }

    fn process_char(&mut self, ch: char) -> Option<TriggerEvent> {
        let prefix_char_0 = self.prefix.chars().next().unwrap_or(';');
        let prefix_char_1 = self.prefix.chars().nth(1).unwrap_or(';');

        match self.state {
            DetectorState::Idle => {
                if ch == prefix_char_0 {
                    self.state = DetectorState::Prefix1;
                    self.buffer.clear();
                    self.buffer.push(ch);
                    log::info!("[STATE] Idle -> Prefix1 | buffer: \"{}\"", self.buffer);
                }
                None
            }
            DetectorState::Prefix1 => {
                self.buffer.push(ch);
                if ch == prefix_char_1 {
                    self.state = DetectorState::Prefix2;
                    log::info!("[STATE] Prefix1 -> Prefix2 | buffer: \"{}\"", self.buffer);
                } else {
                    log::info!("[STATE] Prefix1 -> Idle (wrong char: '{}') | resetting", ch);
                    self.reset();
                }
                None
            }
            DetectorState::Prefix2 => {
                self.buffer.push(ch);
                if is_valid_site_char(ch) {
                    self.state = DetectorState::CollectingSite;
                    log::info!("[STATE] Prefix2 -> CollectingSite | buffer: \"{}\"", self.buffer);
                } else {
                    log::info!("[STATE] Prefix2 -> Idle (invalid char: '{}') | resetting", ch);
                    self.reset();
                }
                None
            }
            DetectorState::CollectingSite => {
                if is_valid_site_char(ch) {
                    self.buffer.push(ch);
                    log::info!("[COLLECT] buffer: \"{}\"", self.buffer);
                } else {
                    log::info!("[STATE] CollectingSite -> Idle (invalid char: '{}') | resetting", ch);
                    self.reset();
                }
                None
            }
        }
    }

    fn handle_terminator(&mut self) -> Option<TriggerEvent> {
        if self.state == DetectorState::CollectingSite && self.buffer.len() > self.prefix.len() {
            let site = self.buffer[self.prefix.len()..].to_string();
            let trigger_len = self.buffer.chars().count() + 1;
            log::info!(
                "[TRIGGER] site: \"{}\" | trigger_len: {} | buffer was: \"{}\"",
                site,
                trigger_len,
                self.buffer
            );
            self.reset();
            return Some(TriggerEvent { site, trigger_len });
        }
        log::info!(
            "[TERMINATOR] No trigger (state: {:?}, buffer: \"{}\")",
            self.state,
            self.buffer
        );
        self.reset();
        None
    }

    fn reset(&mut self) {
        self.state = DetectorState::Idle;
        self.buffer.clear();
    }

    fn handle_backspace(&mut self) {
        if !self.buffer.is_empty() && self.state != DetectorState::Idle {
            self.buffer.pop();
            if self.buffer.len() < self.prefix.len() {
                self.reset();
            }
        }
    }
}

fn is_valid_site_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_'
}

fn key_to_char(key: Key) -> Option<char> {
    match key {
        Key::KeyA => Some('a'),
        Key::KeyB => Some('b'),
        Key::KeyC => Some('c'),
        Key::KeyD => Some('d'),
        Key::KeyE => Some('e'),
        Key::KeyF => Some('f'),
        Key::KeyG => Some('g'),
        Key::KeyH => Some('h'),
        Key::KeyI => Some('i'),
        Key::KeyJ => Some('j'),
        Key::KeyK => Some('k'),
        Key::KeyL => Some('l'),
        Key::KeyM => Some('m'),
        Key::KeyN => Some('n'),
        Key::KeyO => Some('o'),
        Key::KeyP => Some('p'),
        Key::KeyQ => Some('q'),
        Key::KeyR => Some('r'),
        Key::KeyS => Some('s'),
        Key::KeyT => Some('t'),
        Key::KeyU => Some('u'),
        Key::KeyV => Some('v'),
        Key::KeyW => Some('w'),
        Key::KeyX => Some('x'),
        Key::KeyY => Some('y'),
        Key::KeyZ => Some('z'),
        Key::Num0 => Some('0'),
        Key::Num1 => Some('1'),
        Key::Num2 => Some('2'),
        Key::Num3 => Some('3'),
        Key::Num4 => Some('4'),
        Key::Num5 => Some('5'),
        Key::Num6 => Some('6'),
        Key::Num7 => Some('7'),
        Key::Num8 => Some('8'),
        Key::Num9 => Some('9'),
        Key::Dot => Some('.'),
        Key::Minus => Some('-'),
        Key::SemiColon => Some(';'),
        Key::Equal => Some('='),
        Key::Comma => Some(','),
        Key::Slash => Some('/'),
        Key::BackSlash => Some('\\'),
        Key::LeftBracket => Some('['),
        Key::RightBracket => Some(']'),
        Key::Quote => Some('\''),
        Key::BackQuote => Some('`'),
        _ => None,
    }
}

fn is_terminator(key: Key) -> bool {
    matches!(key, Key::Space | Key::Return | Key::Tab)
}

pub fn start_keyboard_listener(
    tx: Sender<TriggerEvent>,
    prefix: String,
    injection_active: Arc<AtomicBool>,
) -> Result<thread::JoinHandle<()>> {
    let handle = thread::spawn(move || {
        let mut detector = TriggerDetector::new(prefix.clone(), injection_active);

        log::info!("[LISTENER] Keyboard listener started, prefix: \"{}\"", prefix);

        let callback = move |event: Event| {
            if let Some(trigger) = detector.process_event(&event) {
                log::info!("[SEND] Sending trigger event: {:?}", trigger);
                if let Err(e) = tx.send(trigger) {
                    log::error!("[ERROR] Failed to send trigger: {}", e);
                }
            }
        };

        log::info!("[LISTENER] Starting rdev::listen...");
        if let Err(e) = listen(callback) {
            log::error!("[ERROR] Keyboard listener error: {:?}", e);
        }
        log::info!("[LISTENER] rdev::listen exited");
    });

    Ok(handle)
}
