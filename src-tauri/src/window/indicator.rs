use tauri::WebviewWindow;

/// Set up the indicator window with special properties
#[allow(unused_variables)]
pub fn setup_indicator_window(window: &WebviewWindow) -> Result<(), String> {
    // Open devtools for debugging in dev mode
    #[cfg(debug_assertions)]
    window.open_devtools();

    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::NSWindowCollectionBehavior;
        use cocoa::base::id;

        let ns_window = window.ns_window().map_err(|e| e.to_string())? as id;

        unsafe {
            // Make window ignore mouse events (click-through)
            use objc::*;
            let _: () = msg_send![ns_window, setIgnoresMouseEvents: true];

            // Set window level to floating
            let _: () = msg_send![ns_window, setLevel: 3i64]; // NSFloatingWindowLevel

            // Set collection behavior to appear on all spaces
            #[allow(deprecated)]
            {
                use cocoa::appkit::NSWindow;
                ns_window.setCollectionBehavior_(
                    NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                        | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary,
                );
            }
        }
    }

    Ok(())
}
