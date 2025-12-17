//! Browser scripting via AppleScript to get focused element positions in web browsers

use super::accessibility::ElementFrame;
use std::process::Command;

/// Supported browser types for AppleScript scripting
#[derive(Debug, Clone, Copy)]
pub enum BrowserType {
    Safari,
    Chrome,
    Arc,
}

impl BrowserType {
    /// Get the application name for AppleScript
    fn app_name(&self) -> &'static str {
        match self {
            BrowserType::Safari => "Safari",
            BrowserType::Chrome => "Google Chrome",
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
        CHROME_BUNDLE | BRAVE_BUNDLE | EDGE_BUNDLE => Some(BrowserType::Chrome),
        ARC_BUNDLE => Some(BrowserType::Arc),
        _ => None,
    }
}

/// JavaScript to get the focused element's bounding rect in screen coordinates
const GET_ELEMENT_RECT_JS: &str = r#"
(function() {
    var el = document.activeElement;
    if (!el || el === document.body || el === document.documentElement) return null;

    // Handle iframes - try to get the actual focused element inside
    if (el.tagName === 'IFRAME') {
        try {
            var iframeDoc = el.contentDocument || el.contentWindow.document;
            if (iframeDoc && iframeDoc.activeElement && iframeDoc.activeElement !== iframeDoc.body) {
                // For iframes, we need the iframe's position plus the element's position within
                var iframeRect = el.getBoundingClientRect();
                var innerEl = iframeDoc.activeElement;
                var innerRect = innerEl.getBoundingClientRect();
                var chromeHeight = window.outerHeight - window.innerHeight;
                return JSON.stringify({
                    x: Math.round(iframeRect.left + innerRect.left + window.screenX),
                    y: Math.round(iframeRect.top + innerRect.top + window.screenY + chromeHeight),
                    width: Math.round(innerRect.width),
                    height: Math.round(innerRect.height)
                });
            }
        } catch(e) {
            // Cross-origin iframe, fall through to use iframe's rect
        }
    }

    // Handle shadow DOM
    if (el.shadowRoot && el.shadowRoot.activeElement) {
        el = el.shadowRoot.activeElement;
    }

    var rect = el.getBoundingClientRect();

    // Skip if element has no dimensions (hidden or not rendered)
    if (rect.width === 0 && rect.height === 0) return null;

    // Calculate browser chrome height (toolbars, tabs, etc.)
    var chromeHeight = window.outerHeight - window.innerHeight;

    return JSON.stringify({
        x: Math.round(rect.left + window.screenX),
        y: Math.round(rect.top + window.screenY + chromeHeight),
        width: Math.round(rect.width),
        height: Math.round(rect.height)
    });
})()
"#;

/// Get the focused element frame from a browser using AppleScript
pub fn get_browser_element_frame(browser_type: BrowserType) -> Option<ElementFrame> {
    log::info!(
        "Attempting to get element frame from browser: {:?}",
        browser_type
    );

    let script = match browser_type {
        BrowserType::Safari => build_safari_script(),
        BrowserType::Chrome | BrowserType::Arc => build_chrome_script(browser_type.app_name()),
    };

    execute_applescript_and_parse(&script)
}

/// Build AppleScript for Safari
fn build_safari_script() -> String {
    // Escape the JavaScript for AppleScript string
    let escaped_js = GET_ELEMENT_RECT_JS
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ");

    format!(
        r#"
        tell application "Safari"
            if (count of windows) = 0 then return "null"
            tell front window
                if (count of tabs) = 0 then return "null"
                set activeTab to current tab
                try
                    set jsResult to do JavaScript "{}" in activeTab
                    return jsResult
                on error
                    return "null"
                end try
            end tell
        end tell
    "#,
        escaped_js
    )
}

/// Build AppleScript for Chrome-based browsers (Chrome, Arc, Brave, Edge)
fn build_chrome_script(app_name: &str) -> String {
    // Escape the JavaScript for AppleScript string
    let escaped_js = GET_ELEMENT_RECT_JS
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ");

    format!(
        r#"
        tell application "{}"
            if (count of windows) = 0 then return "null"
            tell front window
                set activeTab to active tab
                try
                    set jsResult to execute javascript "{}" in activeTab
                    return jsResult
                on error
                    return "null"
                end try
            end tell
        end tell
    "#,
        app_name, escaped_js
    )
}

/// Execute AppleScript and parse the JSON result into ElementFrame
fn execute_applescript_and_parse(script: &str) -> Option<ElementFrame> {
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
    parse_element_frame_json(&stdout)
}

/// Parse JSON string into ElementFrame
fn parse_element_frame_json(json: &str) -> Option<ElementFrame> {
    // Simple JSON parsing without serde dependency
    // Expected format: {"x":123,"y":456,"width":789,"height":100}

    let json = json.trim().trim_matches('"');

    let x = extract_json_number(json, "x")?;
    let y = extract_json_number(json, "y")?;
    let width = extract_json_number(json, "width")?;
    let height = extract_json_number(json, "height")?;

    log::info!(
        "Parsed browser element frame: x={}, y={}, w={}, h={}",
        x,
        y,
        width,
        height
    );

    Some(ElementFrame {
        x,
        y,
        width,
        height,
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
            Some(BrowserType::Chrome)
        ));
        assert!(detect_browser_type("org.mozilla.firefox").is_none());
        assert!(detect_browser_type("com.apple.TextEdit").is_none());
    }

    #[test]
    fn test_parse_element_frame_json() {
        let json = r#"{"x":100,"y":200,"width":300,"height":50}"#;
        let frame = parse_element_frame_json(json).unwrap();
        assert_eq!(frame.x, 100.0);
        assert_eq!(frame.y, 200.0);
        assert_eq!(frame.width, 300.0);
        assert_eq!(frame.height, 50.0);
    }

    #[test]
    fn test_extract_json_number() {
        let json = r#"{"x":123,"y":456}"#;
        assert_eq!(extract_json_number(json, "x"), Some(123.0));
        assert_eq!(extract_json_number(json, "y"), Some(456.0));
        assert_eq!(extract_json_number(json, "z"), None);
    }
}
