use std::ffi::c_void;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};

use super::inject::INJECTED_EVENT_MARKER;
use super::keycode::{KeyEvent, Modifiers};

pub type KeyEventCallback = Box<dyn Fn(KeyEvent) -> Option<KeyEvent> + Send + 'static>;

// Raw C types for CGEventTap (the core-graphics crate has a bug where returning None
// doesn't actually suppress events - it returns the original event instead of null)
type CGEventRef = *mut c_void;
type CGEventTapProxy = *mut c_void;
type CFMachPortRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;

type CGEventTapCallBack = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: CGEventTapCallBack,
        user_info: *mut c_void,
    ) -> CFMachPortRef;

    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    fn CGEventGetFlags(event: CGEventRef) -> u64;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: CFMachPortRef,
        order: i64,
    ) -> CFRunLoopSourceRef;

    fn CFRunLoopAddSource(run_loop: *const c_void, source: CFRunLoopSourceRef, mode: *const c_void);

    fn CFRunLoopGetCurrent() -> *const c_void;
}

const kCGHIDEventTap: u32 = 0;
const kCGHeadInsertEventTap: u32 = 0;
const kCGEventTapOptionDefault: u32 = 0;

const kCGEventKeyDown: u64 = 1 << 10;
const kCGEventKeyUp: u64 = 1 << 11;
const kCGEventFlagsChanged: u64 = 1 << 12;

const kCGKeyboardEventKeycode: u32 = 9;
const kCGEventSourceUserData: u32 = 42;

// Event type constants
const EVENT_TYPE_KEY_DOWN: u32 = 10;
const EVENT_TYPE_KEY_UP: u32 = 11;
const EVENT_TYPE_FLAGS_CHANGED: u32 = 12;
const EVENT_TYPE_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFFFFFE;
const EVENT_TYPE_TAP_DISABLED_BY_USER: u32 = 0xFFFFFFFF;

/// Context passed to the callback
struct CallbackContext {
    callback: Arc<Mutex<Option<KeyEventCallback>>>,
    needs_reenable: Arc<AtomicBool>,
}

// Global callback context (needed because C callbacks can't capture Rust closures)
static mut CALLBACK_CONTEXT: Option<*mut CallbackContext> = None;

/// Raw C callback function
unsafe extern "C" fn event_callback(
    _proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    if event.is_null() {
        return event;
    }

    let ctx = match CALLBACK_CONTEXT {
        Some(ptr) => &*ptr,
        None => return event,
    };

    // Handle tap disabled events
    if event_type == EVENT_TYPE_TAP_DISABLED_BY_TIMEOUT {
        log::warn!("CGEventTap was disabled by timeout, signaling re-enable...");
        ctx.needs_reenable.store(true, Ordering::SeqCst);
        return event;
    }

    if event_type == EVENT_TYPE_TAP_DISABLED_BY_USER {
        log::warn!("CGEventTap was disabled by user input, signaling re-enable...");
        ctx.needs_reenable.store(true, Ordering::SeqCst);
        return event;
    }

    // Skip events we injected ourselves
    let user_data = CGEventGetIntegerValueField(event, kCGEventSourceUserData);
    if user_data == INJECTED_EVENT_MARKER {
        log::trace!("Skipping injected event");
        return event;
    }

    // Skip FlagsChanged events (modifier key changes) - pass through
    if event_type == EVENT_TYPE_FLAGS_CHANGED {
        return event;
    }

    // Only process KeyDown and KeyUp
    if event_type != EVENT_TYPE_KEY_DOWN && event_type != EVENT_TYPE_KEY_UP {
        return event;
    }

    let keycode = CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode) as u16;
    let flags = CGEventGetFlags(event);
    let is_key_down = event_type == EVENT_TYPE_KEY_DOWN;

    log::trace!("Key event: keycode={}, type={}", keycode, event_type);

    let key_event = KeyEvent {
        code: keycode,
        modifiers: Modifiers::from_cg_flags(flags),
        is_key_down,
    };

    // Call user callback
    let cb_lock = ctx.callback.lock().unwrap();
    if let Some(ref cb) = *cb_lock {
        match cb(key_event) {
            Some(_modified_event) => {
                log::trace!("capture: passing through keycode={}", keycode);
                event
            }
            None => {
                // Suppress the event by returning NULL
                log::trace!("capture: SUPPRESSING keycode={}", keycode);
                ptr::null_mut()
            }
        }
    } else {
        log::trace!("capture: no callback, passing through keycode={}", keycode);
        event
    }
}

/// Keyboard capture using raw CGEventTap API
/// (The core-graphics crate has a bug where returning None doesn't suppress events)
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
        let needs_reenable = Arc::new(AtomicBool::new(false));

        thread::spawn(move || {
            unsafe {
                // Create callback context
                let ctx = Box::new(CallbackContext {
                    callback,
                    needs_reenable: Arc::clone(&needs_reenable),
                });
                let ctx_ptr = Box::into_raw(ctx);
                CALLBACK_CONTEXT = Some(ctx_ptr);

                let event_mask = kCGEventKeyDown | kCGEventKeyUp | kCGEventFlagsChanged;

                // Create event tap at HID level for reliable suppression
                let tap = CGEventTapCreate(
                    kCGHIDEventTap,
                    kCGHeadInsertEventTap,
                    kCGEventTapOptionDefault,
                    event_mask,
                    event_callback,
                    ptr::null_mut(),
                );

                if tap.is_null() {
                    log::error!(
                        "Failed to create CGEventTap. Make sure Input Monitoring permission is granted."
                    );
                    *running_flag.lock().unwrap() = false;
                    // Clean up context
                    let _ = Box::from_raw(ctx_ptr);
                    CALLBACK_CONTEXT = None;
                    return;
                }

                // Create run loop source
                let source = CFMachPortCreateRunLoopSource(ptr::null(), tap, 0);
                if source.is_null() {
                    log::error!("Failed to create run loop source!");
                    *running_flag.lock().unwrap() = false;
                    let _ = Box::from_raw(ctx_ptr);
                    CALLBACK_CONTEXT = None;
                    return;
                }

                // Add to run loop
                let run_loop = CFRunLoopGetCurrent();
                CFRunLoopAddSource(run_loop, source, kCFRunLoopDefaultMode as *const c_void);

                // Enable the tap
                CGEventTapEnable(tap, true);

                log::info!("CGEventTap started successfully");

                // Run the loop
                while *running_flag.lock().unwrap() {
                    // Check if tap needs re-enabling
                    if needs_reenable.swap(false, Ordering::SeqCst) {
                        log::info!("Re-enabling CGEventTap...");
                        CGEventTapEnable(tap, true);
                    }

                    CFRunLoop::run_in_mode(
                        kCFRunLoopDefaultMode,
                        Duration::from_millis(100),
                        false,
                    );
                }

                log::info!("CGEventTap stopped");

                // Clean up context
                let _ = Box::from_raw(ctx_ptr);
                CALLBACK_CONTEXT = None;
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
