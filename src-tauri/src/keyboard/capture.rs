use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions,
    CGEventTapPlacement, CGEventTapProxy, CGEventType, EventField,
};

use super::inject::INJECTED_EVENT_MARKER;
use super::keycode::{KeyEvent, Modifiers};

pub type KeyEventCallback = Box<dyn Fn(KeyEvent) -> Option<KeyEvent> + Send + 'static>;

/// Helper to compare CGEventType (which doesn't implement PartialEq)
fn is_event_type(event_type: CGEventType, expected: CGEventType) -> bool {
    // Compare by converting to u32
    (event_type as u32) == (expected as u32)
}

/// Keyboard capture using CGEventTap
pub struct KeyboardCapture {
    callback: Arc<Mutex<Option<KeyEventCallback>>>,
    running: Arc<Mutex<bool>>,
}

impl KeyboardCapture {
    pub fn new() -> Self {
        Self {
            callback: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Set the callback for key events
    /// Return Some(event) to pass through (possibly modified)
    /// Return None to suppress the event
    pub fn set_callback<F>(&self, callback: F)
    where
        F: Fn(KeyEvent) -> Option<KeyEvent> + Send + 'static,
    {
        let mut cb = self.callback.lock().unwrap();
        *cb = Some(Box::new(callback));
    }

    /// Start capturing keyboard events
    /// This spawns a new thread with its own run loop
    pub fn start(&self) -> Result<(), String> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        let callback = Arc::clone(&self.callback);
        let running_flag = Arc::clone(&self.running);

        thread::spawn(move || {
            // Create the event tap with the proper API
            let tap = CGEventTap::new(
                CGEventTapLocation::Session,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::Default,
                vec![
                    CGEventType::KeyDown,
                    CGEventType::KeyUp,
                    CGEventType::FlagsChanged,
                ],
                |_proxy: CGEventTapProxy, event_type: CGEventType, event: &CGEvent| -> Option<CGEvent> {
                    // Handle tap disabled by timeout - re-enable
                    if is_event_type(event_type, CGEventType::TapDisabledByTimeout) {
                        log::warn!("CGEventTap was disabled by timeout, re-enabling...");
                        return Some(event.clone());
                    }

                    // Handle tap disabled by user
                    if is_event_type(event_type, CGEventType::TapDisabledByUserInput) {
                        log::warn!("CGEventTap was disabled by user input");
                        return Some(event.clone());
                    }

                    // Skip events we injected ourselves
                    let user_data = event.get_integer_value_field(EventField::EVENT_SOURCE_USER_DATA);
                    if user_data == INJECTED_EVENT_MARKER {
                        log::trace!("Skipping injected event");
                        return Some(event.clone());
                    }

                    // Skip FlagsChanged events (modifier key changes) - pass through
                    if is_event_type(event_type, CGEventType::FlagsChanged) {
                        return Some(event.clone());
                    }

                    // Get key code and flags
                    let keycode = event.get_integer_value_field(core_graphics::event::EventField::KEYBOARD_EVENT_KEYCODE) as u16;
                    log::trace!("Key event: keycode={}, type={:?}", keycode, event_type);
                    let flags = event.get_flags();
                    let is_key_down = is_event_type(event_type, CGEventType::KeyDown);

                    let key_event = KeyEvent {
                        code: keycode,
                        modifiers: Modifiers::from_cg_flags(flags.bits()),
                        is_key_down,
                    };

                    // Call user callback
                    let cb_lock = callback.lock().unwrap();
                    if let Some(ref cb) = *cb_lock {
                        match cb(key_event) {
                            Some(_modified_event) => {
                                // Pass through
                                Some(event.clone())
                            }
                            None => {
                                // Suppress the event
                                None
                            }
                        }
                    } else {
                        // No callback set, pass through
                        Some(event.clone())
                    }
                },
            );

            match tap {
                Ok(tap) => {
                    // Create run loop source and add to run loop
                    let loop_source = tap
                        .mach_port
                        .create_runloop_source(0)
                        .expect("Failed to create run loop source");

                    let run_loop = CFRunLoop::get_current();
                    unsafe {
                        run_loop.add_source(&loop_source, kCFRunLoopDefaultMode);
                    }

                    // Enable the tap
                    tap.enable();

                    log::info!("CGEventTap started successfully");

                    // Run the loop - use kCFRunLoopDefaultMode, not kCFRunLoopCommonModes
                    while *running_flag.lock().unwrap() {
                        CFRunLoop::run_in_mode(
                            unsafe { kCFRunLoopDefaultMode },
                            Duration::from_millis(100),
                            false,
                        );
                    }

                    log::info!("CGEventTap stopped");
                }
                Err(()) => {
                    log::error!(
                        "Failed to create CGEventTap. Make sure Input Monitoring permission is granted."
                    );
                    *running_flag.lock().unwrap() = false;
                }
            }
        });

        Ok(())
    }

    /// Stop capturing keyboard events
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }

    /// Check if currently capturing
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }
}

impl Default for KeyboardCapture {
    fn default() -> Self {
        Self::new()
    }
}
