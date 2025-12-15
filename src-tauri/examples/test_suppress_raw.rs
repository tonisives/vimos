// Test key suppression using raw C API
// Run with: cargo run --example test_suppress_raw

use core_foundation::base::TCFType;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType};
use std::ffi::c_void;
use std::ptr;

// Raw C types and functions
type CGEventRef = *mut c_void;
type CGEventTapProxy = *mut c_void;
type CFMachPortRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;

type CGEventTapCallBack = extern "C" fn(
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
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: CFMachPortRef,
        order: i64,
    ) -> CFRunLoopSourceRef;

    fn CFRunLoopAddSource(
        run_loop: *const c_void,
        source: CFRunLoopSourceRef,
        mode: *const c_void,
    );

    fn CFRunLoopGetCurrent() -> *const c_void;

    fn CFRunLoopRun();
}

const kCGSessionEventTap: u32 = 1;
const kCGHIDEventTap: u32 = 0;
const kCGHeadInsertEventTap: u32 = 0;
const kCGEventTapOptionDefault: u32 = 0;

const kCGEventKeyDown: u64 = 1 << 10;
const kCGEventKeyUp: u64 = 1 << 11;

const kCGKeyboardEventKeycode: u32 = 9;

// Callback function
extern "C" fn event_callback(
    _proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    if event.is_null() {
        return event;
    }

    let keycode = unsafe { CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode) } as u16;

    let type_str = match event_type {
        10 => "KeyDown",
        11 => "KeyUp",
        _ => "Other",
    };

    // Suppress 'b' key (keycode 11)
    if keycode == 11 {
        println!("[RAW SUPPRESS] keycode={} ({}) - returning NULL", keycode, type_str);
        return ptr::null_mut(); // Return NULL to suppress
    }

    println!("[RAW PASS] keycode={} ({}) - returning event", keycode, type_str);
    event
}

fn main() {
    println!("Raw key suppression test - press 'b' to test suppression");
    println!("Press Ctrl+C to exit");
    println!();

    unsafe {
        let event_mask = kCGEventKeyDown | kCGEventKeyUp;

        // Try HID tap location first
        let tap = CGEventTapCreate(
            kCGHIDEventTap,
            kCGHeadInsertEventTap,
            kCGEventTapOptionDefault,
            event_mask,
            event_callback,
            ptr::null_mut(),
        );

        if tap.is_null() {
            eprintln!("Failed to create event tap at HID level, trying Session...");

            let tap = CGEventTapCreate(
                kCGSessionEventTap,
                kCGHeadInsertEventTap,
                kCGEventTapOptionDefault,
                event_mask,
                event_callback,
                ptr::null_mut(),
            );

            if tap.is_null() {
                eprintln!("Failed to create event tap!");
                eprintln!("Make sure you have Accessibility permissions granted.");
                std::process::exit(1);
            }
        }

        println!("Event tap created successfully!");

        // Create run loop source
        let source = CFMachPortCreateRunLoopSource(ptr::null(), tap, 0);
        if source.is_null() {
            eprintln!("Failed to create run loop source!");
            std::process::exit(1);
        }

        // Add to run loop
        let run_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes as *const c_void);

        // Enable the tap
        CGEventTapEnable(tap, true);

        println!("Event tap enabled!");
        println!();

        // Run the loop
        CFRunLoopRun();
    }
}
