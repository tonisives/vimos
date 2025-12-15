// Test different key suppression methods
// Run with: cargo run --example test_suppress

use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
    CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventTapProxy, CGEventType, EventField,
};
use std::time::Duration;

fn main() {
    println!("Key suppression test - press 'b' to test suppression");
    println!("Press Ctrl+C to exit");
    println!();

    // Try different tap locations
    let locations = [
        ("HID", CGEventTapLocation::HID),
        ("Session", CGEventTapLocation::Session),
        ("AnnotatedSession", CGEventTapLocation::AnnotatedSession),
    ];

    // Use Session location (most common for user-space apps)
    let location = CGEventTapLocation::HID;
    println!("Using tap location: HID");

    let tap = CGEventTap::new(
        location,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default, // Must be Default (not ListenOnly) to suppress
        vec![CGEventType::KeyDown, CGEventType::KeyUp],
        move |_proxy: CGEventTapProxy,
              event_type: CGEventType,
              event: &CGEvent|
              -> Option<CGEvent> {
            let keycode =
                event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;

            // Only process KeyDown and KeyUp
            let is_key_down = matches!(event_type, CGEventType::KeyDown);
            let event_type_str = if is_key_down { "KeyDown" } else { "KeyUp" };

            // Suppress 'b' key (keycode 11)
            if keycode == 11 {
                println!(
                    "[SUPPRESS] keycode={} ({}) - returning None",
                    keycode, event_type_str
                );
                return None; // This should suppress the event
            }

            // Pass through all other keys
            println!(
                "[PASS] keycode={} ({}) - returning Some(event)",
                keycode, event_type_str
            );
            Some(event.clone())
        },
    );

    match tap {
        Ok(tap) => {
            let loop_source = tap
                .mach_port
                .create_runloop_source(0)
                .expect("Failed to create run loop source");

            let run_loop = CFRunLoop::get_current();
            unsafe {
                run_loop.add_source(&loop_source, kCFRunLoopDefaultMode);
            }

            tap.enable();
            println!("Event tap enabled successfully!");
            println!();

            // Run the event loop
            loop {
                CFRunLoop::run_in_mode(
                    unsafe { kCFRunLoopDefaultMode },
                    Duration::from_millis(100),
                    false,
                );
            }
        }
        Err(()) => {
            eprintln!("Failed to create event tap!");
            eprintln!("Make sure you have Accessibility permissions granted.");
            std::process::exit(1);
        }
    }
}
