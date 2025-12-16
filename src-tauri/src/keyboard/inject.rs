use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, EventField};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

use super::keycode::{KeyCode, Modifiers};

/// Custom user data field to mark our injected events
/// We use a high value that's unlikely to conflict with real keycodes
pub const INJECTED_EVENT_MARKER: i64 = 0x54495649; // "TIVI" in hex

/// Inject a single key event
pub fn inject_key(keycode: KeyCode, key_down: bool, modifiers: Modifiers) -> Result<(), String> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| "Failed to create event source")?;

    let event = CGEvent::new_keyboard_event(source, keycode.as_raw(), key_down)
        .map_err(|_| "Failed to create keyboard event")?;

    let flags = CGEventFlags::from_bits_truncate(modifiers.to_cg_flags());
    event.set_flags(flags);

    // Mark the event as injected by us so we don't capture it again
    event.set_integer_value_field(EventField::EVENT_SOURCE_USER_DATA, INJECTED_EVENT_MARKER);

    event.post(CGEventTapLocation::HID);

    Ok(())
}

/// Inject a key press (down + up)
pub fn inject_key_press(keycode: KeyCode, modifiers: Modifiers) -> Result<(), String> {
    inject_key(keycode, true, modifiers)?;
    inject_key(keycode, false, modifiers)?;
    Ok(())
}

/// Inject arrow key with optional modifiers
pub fn inject_arrow(direction: ArrowDirection, modifiers: Modifiers) -> Result<(), String> {
    let keycode = match direction {
        ArrowDirection::Left => KeyCode::Left,
        ArrowDirection::Right => KeyCode::Right,
        ArrowDirection::Up => KeyCode::Up,
        ArrowDirection::Down => KeyCode::Down,
    };
    inject_key_press(keycode, modifiers)
}

#[derive(Debug, Clone, Copy)]
pub enum ArrowDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Move cursor left (h)
pub fn cursor_left(count: u32, select: bool) -> Result<(), String> {
    let mods = if select {
        Modifiers { shift: true, ..Default::default() }
    } else {
        Modifiers::default()
    };

    for _ in 0..count {
        inject_arrow(ArrowDirection::Left, mods)?;
    }
    Ok(())
}

/// Move cursor right (l)
pub fn cursor_right(count: u32, select: bool) -> Result<(), String> {
    let mods = if select {
        Modifiers { shift: true, ..Default::default() }
    } else {
        Modifiers::default()
    };

    for _ in 0..count {
        inject_arrow(ArrowDirection::Right, mods)?;
    }
    Ok(())
}

/// Move cursor up (k)
pub fn cursor_up(count: u32, select: bool) -> Result<(), String> {
    let mods = if select {
        Modifiers { shift: true, ..Default::default() }
    } else {
        Modifiers::default()
    };

    for _ in 0..count {
        inject_arrow(ArrowDirection::Up, mods)?;
    }
    Ok(())
}

/// Move cursor down (j)
pub fn cursor_down(count: u32, select: bool) -> Result<(), String> {
    let mods = if select {
        Modifiers { shift: true, ..Default::default() }
    } else {
        Modifiers::default()
    };

    for _ in 0..count {
        inject_arrow(ArrowDirection::Down, mods)?;
    }
    Ok(())
}

/// Move to start of word (b) - Option+Left on macOS
pub fn word_backward(count: u32, select: bool) -> Result<(), String> {
    let mods = Modifiers {
        option: true,
        shift: select,
        ..Default::default()
    };

    for _ in 0..count {
        inject_arrow(ArrowDirection::Left, mods)?;
    }
    Ok(())
}

/// Move to end of word (e) / next word (w) - Option+Right on macOS
pub fn word_forward(count: u32, select: bool) -> Result<(), String> {
    let mods = Modifiers {
        option: true,
        shift: select,
        ..Default::default()
    };

    for _ in 0..count {
        inject_arrow(ArrowDirection::Right, mods)?;
    }
    Ok(())
}

/// Move to start of line (0/^) - Cmd+Left on macOS
pub fn line_start(select: bool) -> Result<(), String> {
    let mods = Modifiers {
        command: true,
        shift: select,
        ..Default::default()
    };
    inject_arrow(ArrowDirection::Left, mods)
}

/// Move to end of line ($) - Cmd+Right on macOS
pub fn line_end(select: bool) -> Result<(), String> {
    let mods = Modifiers {
        command: true,
        shift: select,
        ..Default::default()
    };
    inject_arrow(ArrowDirection::Right, mods)
}

/// Move to start of document (gg) - Cmd+Up on macOS
pub fn document_start(select: bool) -> Result<(), String> {
    let mods = Modifiers {
        command: true,
        shift: select,
        ..Default::default()
    };
    inject_arrow(ArrowDirection::Up, mods)
}

/// Move to end of document (G) - Cmd+Down on macOS
pub fn document_end(select: bool) -> Result<(), String> {
    let mods = Modifiers {
        command: true,
        shift: select,
        ..Default::default()
    };
    inject_arrow(ArrowDirection::Down, mods)
}

/// Page up (Ctrl+b or Ctrl+u)
pub fn page_up(select: bool) -> Result<(), String> {
    let mods = Modifiers {
        shift: select,
        ..Default::default()
    };
    inject_key_press(KeyCode::PageUp, mods)
}

/// Page down (Ctrl+f or Ctrl+d)
pub fn page_down(select: bool) -> Result<(), String> {
    let mods = Modifiers {
        shift: select,
        ..Default::default()
    };
    inject_key_press(KeyCode::PageDown, mods)
}

/// Delete character (x)
pub fn delete_char() -> Result<(), String> {
    inject_key_press(KeyCode::ForwardDelete, Modifiers::default())
}

/// Delete character before cursor (X)
pub fn backspace() -> Result<(), String> {
    inject_key_press(KeyCode::Delete, Modifiers::default())
}

/// Cut selection (Cmd+X)
pub fn cut() -> Result<(), String> {
    inject_key_press(
        KeyCode::X,
        Modifiers {
            command: true,
            ..Default::default()
        },
    )
}

/// Copy selection (Cmd+C)
pub fn copy() -> Result<(), String> {
    inject_key_press(
        KeyCode::C,
        Modifiers {
            command: true,
            ..Default::default()
        },
    )
}

/// Paste (Cmd+V)
pub fn paste() -> Result<(), String> {
    inject_key_press(
        KeyCode::V,
        Modifiers {
            command: true,
            ..Default::default()
        },
    )
}

/// Undo (Cmd+Z)
pub fn undo() -> Result<(), String> {
    inject_key_press(
        KeyCode::Z,
        Modifiers {
            command: true,
            ..Default::default()
        },
    )
}

/// Redo (Cmd+Shift+Z)
pub fn redo() -> Result<(), String> {
    inject_key_press(
        KeyCode::Z,
        Modifiers {
            command: true,
            shift: true,
            ..Default::default()
        },
    )
}

/// New line below (o) - Cmd+Right, Return
pub fn new_line_below() -> Result<(), String> {
    line_end(false)?;
    inject_key_press(KeyCode::Return, Modifiers::default())
}

/// New line above (O) - Cmd+Left, Return, Up
pub fn new_line_above() -> Result<(), String> {
    line_start(false)?;
    inject_key_press(KeyCode::Return, Modifiers::default())?;
    cursor_up(1, false)
}

/// Paragraph up ({) - Option+Up on macOS
pub fn paragraph_up(count: u32, select: bool) -> Result<(), String> {
    let mods = Modifiers {
        option: true,
        shift: select,
        ..Default::default()
    };

    for _ in 0..count {
        inject_arrow(ArrowDirection::Up, mods)?;
    }
    Ok(())
}

/// Paragraph down (}) - Option+Down on macOS
pub fn paragraph_down(count: u32, select: bool) -> Result<(), String> {
    let mods = Modifiers {
        option: true,
        shift: select,
        ..Default::default()
    };

    for _ in 0..count {
        inject_arrow(ArrowDirection::Down, mods)?;
    }
    Ok(())
}

/// Join lines (J) - go to end, delete newline, add space
pub fn join_lines() -> Result<(), String> {
    line_end(false)?;
    delete_char()?;
    inject_key_press(KeyCode::Space, Modifiers::default())
}

/// Select inner word (iw) - Option+Left to word start, Option+Shift+Right to select word
pub fn select_inner_word() -> Result<(), String> {
    word_backward(1, false)?;
    word_forward(1, true)
}

/// Select around word (aw) - inner word + trailing space
pub fn select_around_word() -> Result<(), String> {
    word_backward(1, false)?;
    word_forward(1, true)?;
    cursor_right(1, true)
}

/// Indent line (>>) - Tab key
pub fn indent_line() -> Result<(), String> {
    line_start(false)?;
    inject_key_press(KeyCode::Tab, Modifiers::default())
}

/// Outdent line (<<) - Shift+Tab
pub fn outdent_line() -> Result<(), String> {
    line_start(false)?;
    inject_key_press(
        KeyCode::Tab,
        Modifiers {
            shift: true,
            ..Default::default()
        },
    )
}

/// Type a character
pub fn type_char(keycode: KeyCode, shift: bool) -> Result<(), String> {
    let mods = if shift {
        Modifiers {
            shift: true,
            ..Default::default()
        }
    } else {
        Modifiers::default()
    };
    inject_key_press(keycode, mods)
}
