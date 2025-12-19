//! Browser scripting via AppleScript to get focused element positions in web browsers

use super::accessibility::ElementFrame;
use std::process::Command;

/// Supported browser types for AppleScript scripting
#[derive(Debug, Clone, Copy)]
pub enum BrowserType {
    Safari,
    Chrome,
    Brave,
    Arc,
}

impl BrowserType {
    /// Get the application name for AppleScript
    fn app_name(&self) -> &'static str {
        match self {
            BrowserType::Safari => "Safari",
            BrowserType::Chrome => "Google Chrome",
            BrowserType::Brave => "Brave Browser",
            BrowserType::Arc => "Arc",
        }
    }
}

/// Browser bundle ID constants
pub const SAFARI_BUNDLE: &str = "com.apple.Safari";
pub const CHROME_BUNDLE: &str = "com.google.Chrome";
pub const ARC_BUNDLE: &str = "company.thebrowser.Browser";
pub const BRAVE_BUNDLE: &str = "com.brave.Browser";
pub const EDGE_BUNDLE: &str = "com.microsoft.edgemac";

/// Detect if a bundle ID corresponds to a scriptable browser
pub fn detect_browser_type(bundle_id: &str) -> Option<BrowserType> {
    match bundle_id {
        SAFARI_BUNDLE => Some(BrowserType::Safari),
        CHROME_BUNDLE | EDGE_BUNDLE => Some(BrowserType::Chrome),
        BRAVE_BUNDLE => Some(BrowserType::Brave),
        ARC_BUNDLE => Some(BrowserType::Arc),
        _ => None,
    }
}

/// JavaScript to get the focused element's viewport-relative position and viewport height
const GET_ELEMENT_RECT_JS: &str = r#"(function() { var el = document.activeElement; if (!el || el === document.body || el === document.documentElement) return null; if (el.tagName === 'IFRAME') { try { var iframeDoc = el.contentDocument || el.contentWindow.document; if (iframeDoc && iframeDoc.activeElement && iframeDoc.activeElement !== iframeDoc.body) { var iframeRect = el.getBoundingClientRect(); var innerEl = iframeDoc.activeElement; var innerRect = innerEl.getBoundingClientRect(); return JSON.stringify({ x: Math.round(iframeRect.left + innerRect.left), y: Math.round(iframeRect.top + innerRect.top), width: Math.round(innerRect.width), height: Math.round(innerRect.height), viewportHeight: window.innerHeight }); } } catch(e) {} } if (el.shadowRoot && el.shadowRoot.activeElement) { el = el.shadowRoot.activeElement; } var rect = el.getBoundingClientRect(); if (rect.width === 0 && rect.height === 0) return null; return JSON.stringify({ x: Math.round(rect.left), y: Math.round(rect.top), width: Math.round(rect.width), height: Math.round(rect.height), viewportHeight: window.innerHeight }); })()"#;

/// Get the focused element frame from a browser using AppleScript
pub fn get_browser_element_frame(browser_type: BrowserType) -> Option<ElementFrame> {
    log::info!(
        "Attempting to get element frame from browser: {:?}",
        browser_type
    );

    // Get window position and size from System Events
    let (window_x, window_y, _window_width, window_height) = get_browser_window_bounds(browser_type.app_name())?;
    log::info!("Browser window bounds: x={}, y={}, h={}", window_x, window_y, window_height);

    // Get element's viewport-relative position via JavaScript
    let script = match browser_type {
        BrowserType::Safari => build_safari_script(),
        BrowserType::Chrome | BrowserType::Brave | BrowserType::Arc => {
            build_chrome_script(browser_type.app_name())
        }
    };

    let frame = execute_applescript_and_parse(&script)?;

    // Calculate chrome height: window height - viewport height
    let chrome_height = window_height - frame.viewport_height.unwrap_or(window_height);
    log::info!("Chrome height: {} (window_h={}, viewport_h={})", chrome_height, window_height, frame.viewport_height.unwrap_or(0.0));

    // Combine: window position + chrome height + viewport-relative element position
    Some(ElementFrame {
        x: window_x + frame.x,
        y: window_y + chrome_height + frame.y,
        width: frame.width,
        height: frame.height,
    })
}

/// Get the browser window's position and size using System Events
fn get_browser_window_bounds(app_name: &str) -> Option<(f64, f64, f64, f64)> {
    let script = format!(
        r#"tell application "System Events"
    set winPos to position of front window of process "{}"
    set winSize to size of front window of process "{}"
    return (item 1 of winPos as text) & "," & (item 2 of winPos as text) & "," & (item 1 of winSize as text) & "," & (item 2 of winSize as text)
end tell"#,
        app_name, app_name
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .ok()?;

    if !output.status.success() {
        log::warn!("Failed to get window bounds: {}", String::from_utf8_lossy(&output.stderr));
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.trim().split(',').collect();
    if parts.len() != 4 {
        log::warn!("Unexpected window bounds format: {}", stdout);
        return None;
    }

    let x: f64 = parts[0].parse().ok()?;
    let y: f64 = parts[1].parse().ok()?;
    let w: f64 = parts[2].parse().ok()?;
    let h: f64 = parts[3].parse().ok()?;

    Some((x, y, w, h))
}

/// Build AppleScript for Safari
fn build_safari_script() -> String {
    format!(
        r#"tell application "Safari"
    if (count of windows) = 0 then return "null"
    tell front window
        if (count of tabs) = 0 then return "null"
        try
            return do JavaScript "{}" in current tab
        on error
            return "null"
        end try
    end tell
end tell"#,
        GET_ELEMENT_RECT_JS.replace('"', "\\\"")
    )
}

/// Build AppleScript for Chrome-based browsers (Chrome, Arc, Brave, Edge)
fn build_chrome_script(app_name: &str) -> String {
    format!(
        r#"tell application "{}"
    if (count of windows) = 0 then return "null"
    tell active tab of front window
        try
            return execute javascript "{}"
        on error
            return "null"
        end try
    end tell
end tell"#,
        app_name,
        GET_ELEMENT_RECT_JS.replace('"', "\\\"")
    )
}

/// Viewport-relative frame from browser JavaScript
struct ViewportFrame {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    viewport_height: Option<f64>,
}

/// Execute AppleScript and parse the JSON result into ViewportFrame
fn execute_applescript_and_parse(script: &str) -> Option<ViewportFrame> {
    // Execute with timeout
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            log::warn!("Failed to execute AppleScript: {}", e);
            return None;
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("AppleScript failed: {}", stderr);
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    log::info!("AppleScript returned: {}", stdout);

    // Handle "null" or empty response
    if stdout.is_empty() || stdout == "null" || stdout == "missing value" {
        log::info!("Browser returned no element frame");
        return None;
    }

    // Parse JSON
    parse_viewport_frame_json(&stdout)
}

/// Parse JSON string into ViewportFrame
fn parse_viewport_frame_json(json: &str) -> Option<ViewportFrame> {
    // Simple JSON parsing without serde dependency
    // Expected format: {"x":123,"y":456,"width":789,"height":100,"viewportHeight":800}

    let json = json.trim().trim_matches('"');

    let x = extract_json_number(json, "x")?;
    let y = extract_json_number(json, "y")?;
    let width = extract_json_number(json, "width")?;
    let height = extract_json_number(json, "height")?;
    let viewport_height = extract_json_number(json, "viewportHeight");

    log::info!(
        "Parsed viewport frame: x={}, y={}, w={}, h={}, viewport_h={:?}",
        x,
        y,
        width,
        height,
        viewport_height
    );

    Some(ViewportFrame {
        x,
        y,
        width,
        height,
        viewport_height,
    })
}

/// Extract a number from a JSON string by key
fn extract_json_number(json: &str, key: &str) -> Option<f64> {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let remaining = &json[start..];

    // Find the end of the number (comma, }, or end of string)
    let end = remaining
        .find(|c: char| c == ',' || c == '}')
        .unwrap_or(remaining.len());

    let num_str = remaining[..end].trim();
    num_str.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_browser_type() {
        assert!(matches!(
            detect_browser_type("com.apple.Safari"),
            Some(BrowserType::Safari)
        ));
        assert!(matches!(
            detect_browser_type("com.google.Chrome"),
            Some(BrowserType::Chrome)
        ));
        assert!(matches!(
            detect_browser_type("company.thebrowser.Browser"),
            Some(BrowserType::Arc)
        ));
        assert!(matches!(
            detect_browser_type("com.brave.Browser"),
            Some(BrowserType::Brave)
        ));
        assert!(detect_browser_type("org.mozilla.firefox").is_none());
        assert!(detect_browser_type("com.apple.TextEdit").is_none());
    }

    #[test]
    fn test_parse_viewport_frame_json() {
        let json = r#"{"x":100,"y":200,"width":300,"height":50,"viewportHeight":800}"#;
        let frame = parse_viewport_frame_json(json).unwrap();
        assert_eq!(frame.x, 100.0);
        assert_eq!(frame.y, 200.0);
        assert_eq!(frame.width, 300.0);
        assert_eq!(frame.height, 50.0);
        assert_eq!(frame.viewport_height, Some(800.0));
    }

    #[test]
    fn test_extract_json_number() {
        let json = r#"{"x":123,"y":456}"#;
        assert_eq!(extract_json_number(json, "x"), Some(123.0));
        assert_eq!(extract_json_number(json, "y"), Some(456.0));
        assert_eq!(extract_json_number(json, "z"), None);
    }
}
