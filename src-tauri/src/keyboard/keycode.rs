/// macOS virtual keycodes
/// Reference: https://developer.apple.com/documentation/carbon/1430449-virtual_key_codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
#[allow(dead_code)]
pub enum KeyCode {
    // Letters
    A = 0x00,
    S = 0x01,
    D = 0x02,
    F = 0x03,
    H = 0x04,
    G = 0x05,
    Z = 0x06,
    X = 0x07,
    C = 0x08,
    V = 0x09,
    B = 0x0B,
    Q = 0x0C,
    W = 0x0D,
    E = 0x0E,
    R = 0x0F,
    Y = 0x10,
    T = 0x11,
    O = 0x1F,
    U = 0x20,
    I = 0x22,
    P = 0x23,
    L = 0x25,
    J = 0x26,
    K = 0x28,
    N = 0x2D,
    M = 0x2E,

    // Numbers
    Num1 = 0x12,
    Num2 = 0x13,
    Num3 = 0x14,
    Num4 = 0x15,
    Num5 = 0x17,
    Num6 = 0x16,
    Num7 = 0x1A,
    Num8 = 0x1C,
    Num9 = 0x19,
    Num0 = 0x1D,

    // Special keys
    Return = 0x24,
    Tab = 0x30,
    Space = 0x31,
    Delete = 0x33,
    Escape = 0x35,
    Command = 0x37,
    Shift = 0x38,
    CapsLock = 0x39,
    Option = 0x3A,
    Control = 0x3B,
    RightShift = 0x3C,
    RightOption = 0x3D,
    RightControl = 0x3E,
    Function = 0x3F,

    // Arrow keys
    Left = 0x7B,
    Right = 0x7C,
    Down = 0x7D,
    Up = 0x7E,

    // Function keys
    F1 = 0x7A,
    F2 = 0x78,
    F3 = 0x63,
    F4 = 0x76,
    F5 = 0x60,
    F6 = 0x61,
    F7 = 0x62,
    F8 = 0x64,
    F9 = 0x65,
    F10 = 0x6D,
    F11 = 0x67,
    F12 = 0x6F,

    // Navigation
    Home = 0x73,
    End = 0x77,
    PageUp = 0x74,
    PageDown = 0x79,
    ForwardDelete = 0x75,

    // Punctuation
    Equal = 0x18,
    Minus = 0x1B,
    LeftBracket = 0x21,
    RightBracket = 0x1E,
    Quote = 0x27,
    Semicolon = 0x29,
    Backslash = 0x2A,
    Comma = 0x2B,
    Slash = 0x2C,
    Period = 0x2F,
    Grave = 0x32,
}

impl KeyCode {
    pub fn from_raw(code: u16) -> Option<Self> {
        match code {
            0x00 => Some(Self::A),
            0x01 => Some(Self::S),
            0x02 => Some(Self::D),
            0x03 => Some(Self::F),
            0x04 => Some(Self::H),
            0x05 => Some(Self::G),
            0x06 => Some(Self::Z),
            0x07 => Some(Self::X),
            0x08 => Some(Self::C),
            0x09 => Some(Self::V),
            0x0B => Some(Self::B),
            0x0C => Some(Self::Q),
            0x0D => Some(Self::W),
            0x0E => Some(Self::E),
            0x0F => Some(Self::R),
            0x10 => Some(Self::Y),
            0x11 => Some(Self::T),
            0x1F => Some(Self::O),
            0x20 => Some(Self::U),
            0x22 => Some(Self::I),
            0x23 => Some(Self::P),
            0x25 => Some(Self::L),
            0x26 => Some(Self::J),
            0x28 => Some(Self::K),
            0x2D => Some(Self::N),
            0x2E => Some(Self::M),

            0x12 => Some(Self::Num1),
            0x13 => Some(Self::Num2),
            0x14 => Some(Self::Num3),
            0x15 => Some(Self::Num4),
            0x17 => Some(Self::Num5),
            0x16 => Some(Self::Num6),
            0x1A => Some(Self::Num7),
            0x1C => Some(Self::Num8),
            0x19 => Some(Self::Num9),
            0x1D => Some(Self::Num0),

            0x24 => Some(Self::Return),
            0x30 => Some(Self::Tab),
            0x31 => Some(Self::Space),
            0x33 => Some(Self::Delete),
            0x35 => Some(Self::Escape),
            0x37 => Some(Self::Command),
            0x38 => Some(Self::Shift),
            0x39 => Some(Self::CapsLock),
            0x3A => Some(Self::Option),
            0x3B => Some(Self::Control),
            0x3C => Some(Self::RightShift),
            0x3D => Some(Self::RightOption),
            0x3E => Some(Self::RightControl),
            0x3F => Some(Self::Function),

            0x7B => Some(Self::Left),
            0x7C => Some(Self::Right),
            0x7D => Some(Self::Down),
            0x7E => Some(Self::Up),

            0x7A => Some(Self::F1),
            0x78 => Some(Self::F2),
            0x63 => Some(Self::F3),
            0x76 => Some(Self::F4),
            0x60 => Some(Self::F5),
            0x61 => Some(Self::F6),
            0x62 => Some(Self::F7),
            0x64 => Some(Self::F8),
            0x65 => Some(Self::F9),
            0x6D => Some(Self::F10),
            0x67 => Some(Self::F11),
            0x6F => Some(Self::F12),

            0x73 => Some(Self::Home),
            0x77 => Some(Self::End),
            0x74 => Some(Self::PageUp),
            0x79 => Some(Self::PageDown),
            0x75 => Some(Self::ForwardDelete),

            // Punctuation
            0x18 => Some(Self::Equal),
            0x1B => Some(Self::Minus),
            0x21 => Some(Self::LeftBracket),
            0x1E => Some(Self::RightBracket),
            0x27 => Some(Self::Quote),
            0x29 => Some(Self::Semicolon),
            0x2A => Some(Self::Backslash),
            0x2B => Some(Self::Comma),
            0x2C => Some(Self::Slash),
            0x2F => Some(Self::Period),
            0x32 => Some(Self::Grave),

            _ => None,
        }
    }

    pub fn as_raw(&self) -> u16 {
        *self as u16
    }

    /// Convert keycode to a snake_case string name (for settings storage)
    pub fn to_name(&self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
            Self::C => "c",
            Self::D => "d",
            Self::E => "e",
            Self::F => "f",
            Self::G => "g",
            Self::H => "h",
            Self::I => "i",
            Self::J => "j",
            Self::K => "k",
            Self::L => "l",
            Self::M => "m",
            Self::N => "n",
            Self::O => "o",
            Self::P => "p",
            Self::Q => "q",
            Self::R => "r",
            Self::S => "s",
            Self::T => "t",
            Self::U => "u",
            Self::V => "v",
            Self::W => "w",
            Self::X => "x",
            Self::Y => "y",
            Self::Z => "z",
            Self::Num0 => "0",
            Self::Num1 => "1",
            Self::Num2 => "2",
            Self::Num3 => "3",
            Self::Num4 => "4",
            Self::Num5 => "5",
            Self::Num6 => "6",
            Self::Num7 => "7",
            Self::Num8 => "8",
            Self::Num9 => "9",
            Self::Return => "return",
            Self::Tab => "tab",
            Self::Space => "space",
            Self::Delete => "delete",
            Self::Escape => "escape",
            Self::Command => "command",
            Self::Shift => "shift",
            Self::CapsLock => "caps_lock",
            Self::Option => "option",
            Self::Control => "control",
            Self::RightShift => "right_shift",
            Self::RightOption => "right_option",
            Self::RightControl => "right_control",
            Self::Function => "function",
            Self::Left => "left",
            Self::Right => "right",
            Self::Down => "down",
            Self::Up => "up",
            Self::F1 => "f1",
            Self::F2 => "f2",
            Self::F3 => "f3",
            Self::F4 => "f4",
            Self::F5 => "f5",
            Self::F6 => "f6",
            Self::F7 => "f7",
            Self::F8 => "f8",
            Self::F9 => "f9",
            Self::F10 => "f10",
            Self::F11 => "f11",
            Self::F12 => "f12",
            Self::Home => "home",
            Self::End => "end",
            Self::PageUp => "page_up",
            Self::PageDown => "page_down",
            Self::ForwardDelete => "forward_delete",
            Self::Equal => "equal",
            Self::Minus => "minus",
            Self::LeftBracket => "left_bracket",
            Self::RightBracket => "right_bracket",
            Self::Quote => "quote",
            Self::Semicolon => "semicolon",
            Self::Backslash => "backslash",
            Self::Comma => "comma",
            Self::Slash => "slash",
            Self::Period => "period",
            Self::Grave => "grave",
        }
    }

    /// Convert keycode to a human-readable display name
    pub fn to_display_name(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
            Self::H => "H",
            Self::I => "I",
            Self::J => "J",
            Self::K => "K",
            Self::L => "L",
            Self::M => "M",
            Self::N => "N",
            Self::O => "O",
            Self::P => "P",
            Self::Q => "Q",
            Self::R => "R",
            Self::S => "S",
            Self::T => "T",
            Self::U => "U",
            Self::V => "V",
            Self::W => "W",
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
            Self::Num0 => "0",
            Self::Num1 => "1",
            Self::Num2 => "2",
            Self::Num3 => "3",
            Self::Num4 => "4",
            Self::Num5 => "5",
            Self::Num6 => "6",
            Self::Num7 => "7",
            Self::Num8 => "8",
            Self::Num9 => "9",
            Self::Return => "Return",
            Self::Tab => "Tab",
            Self::Space => "Space",
            Self::Delete => "Delete",
            Self::Escape => "Escape",
            Self::Command => "Command",
            Self::Shift => "Shift",
            Self::CapsLock => "Caps Lock",
            Self::Option => "Option",
            Self::Control => "Control",
            Self::RightShift => "Right Shift",
            Self::RightOption => "Right Option",
            Self::RightControl => "Right Control",
            Self::Function => "Function",
            Self::Left => "Left Arrow",
            Self::Right => "Right Arrow",
            Self::Down => "Down Arrow",
            Self::Up => "Up Arrow",
            Self::F1 => "F1",
            Self::F2 => "F2",
            Self::F3 => "F3",
            Self::F4 => "F4",
            Self::F5 => "F5",
            Self::F6 => "F6",
            Self::F7 => "F7",
            Self::F8 => "F8",
            Self::F9 => "F9",
            Self::F10 => "F10",
            Self::F11 => "F11",
            Self::F12 => "F12",
            Self::Home => "Home",
            Self::End => "End",
            Self::PageUp => "Page Up",
            Self::PageDown => "Page Down",
            Self::ForwardDelete => "Forward Delete",
            Self::Equal => "=",
            Self::Minus => "-",
            Self::LeftBracket => "[",
            Self::RightBracket => "]",
            Self::Quote => "'",
            Self::Semicolon => ";",
            Self::Backslash => "\\",
            Self::Comma => ",",
            Self::Slash => "/",
            Self::Period => ".",
            Self::Grave => "`",
        }
    }

    /// Parse a key name string to KeyCode
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "a" => Some(Self::A),
            "b" => Some(Self::B),
            "c" => Some(Self::C),
            "d" => Some(Self::D),
            "e" => Some(Self::E),
            "f" => Some(Self::F),
            "g" => Some(Self::G),
            "h" => Some(Self::H),
            "i" => Some(Self::I),
            "j" => Some(Self::J),
            "k" => Some(Self::K),
            "l" => Some(Self::L),
            "m" => Some(Self::M),
            "n" => Some(Self::N),
            "o" => Some(Self::O),
            "p" => Some(Self::P),
            "q" => Some(Self::Q),
            "r" => Some(Self::R),
            "s" => Some(Self::S),
            "t" => Some(Self::T),
            "u" => Some(Self::U),
            "v" => Some(Self::V),
            "w" => Some(Self::W),
            "x" => Some(Self::X),
            "y" => Some(Self::Y),
            "z" => Some(Self::Z),
            "0" => Some(Self::Num0),
            "1" => Some(Self::Num1),
            "2" => Some(Self::Num2),
            "3" => Some(Self::Num3),
            "4" => Some(Self::Num4),
            "5" => Some(Self::Num5),
            "6" => Some(Self::Num6),
            "7" => Some(Self::Num7),
            "8" => Some(Self::Num8),
            "9" => Some(Self::Num9),
            "return" => Some(Self::Return),
            "tab" => Some(Self::Tab),
            "space" => Some(Self::Space),
            "delete" => Some(Self::Delete),
            "escape" => Some(Self::Escape),
            "command" => Some(Self::Command),
            "shift" => Some(Self::Shift),
            "caps_lock" => Some(Self::CapsLock),
            "option" => Some(Self::Option),
            "control" => Some(Self::Control),
            "right_shift" => Some(Self::RightShift),
            "right_option" => Some(Self::RightOption),
            "right_control" => Some(Self::RightControl),
            "function" => Some(Self::Function),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "down" => Some(Self::Down),
            "up" => Some(Self::Up),
            "f1" => Some(Self::F1),
            "f2" => Some(Self::F2),
            "f3" => Some(Self::F3),
            "f4" => Some(Self::F4),
            "f5" => Some(Self::F5),
            "f6" => Some(Self::F6),
            "f7" => Some(Self::F7),
            "f8" => Some(Self::F8),
            "f9" => Some(Self::F9),
            "f10" => Some(Self::F10),
            "f11" => Some(Self::F11),
            "f12" => Some(Self::F12),
            "home" => Some(Self::Home),
            "end" => Some(Self::End),
            "page_up" => Some(Self::PageUp),
            "page_down" => Some(Self::PageDown),
            "forward_delete" => Some(Self::ForwardDelete),
            "equal" => Some(Self::Equal),
            "minus" => Some(Self::Minus),
            "left_bracket" => Some(Self::LeftBracket),
            "right_bracket" => Some(Self::RightBracket),
            "quote" => Some(Self::Quote),
            "semicolon" => Some(Self::Semicolon),
            "backslash" => Some(Self::Backslash),
            "comma" => Some(Self::Comma),
            "slash" => Some(Self::Slash),
            "period" => Some(Self::Period),
            "grave" => Some(Self::Grave),
            _ => None,
        }
    }

    /// Convert a numeric keycode to its digit value
    pub fn to_digit(self) -> Option<u32> {
        match self {
            Self::Num0 => Some(0),
            Self::Num1 => Some(1),
            Self::Num2 => Some(2),
            Self::Num3 => Some(3),
            Self::Num4 => Some(4),
            Self::Num5 => Some(5),
            Self::Num6 => Some(6),
            Self::Num7 => Some(7),
            Self::Num8 => Some(8),
            Self::Num9 => Some(9),
            _ => None,
        }
    }

    /// Convert keycode to character (for r{char} replacement)
    pub fn to_char(self) -> Option<char> {
        match self {
            Self::A => Some('a'),
            Self::B => Some('b'),
            Self::C => Some('c'),
            Self::D => Some('d'),
            Self::E => Some('e'),
            Self::F => Some('f'),
            Self::G => Some('g'),
            Self::H => Some('h'),
            Self::I => Some('i'),
            Self::J => Some('j'),
            Self::K => Some('k'),
            Self::L => Some('l'),
            Self::M => Some('m'),
            Self::N => Some('n'),
            Self::O => Some('o'),
            Self::P => Some('p'),
            Self::Q => Some('q'),
            Self::R => Some('r'),
            Self::S => Some('s'),
            Self::T => Some('t'),
            Self::U => Some('u'),
            Self::V => Some('v'),
            Self::W => Some('w'),
            Self::X => Some('x'),
            Self::Y => Some('y'),
            Self::Z => Some('z'),
            Self::Num0 => Some('0'),
            Self::Num1 => Some('1'),
            Self::Num2 => Some('2'),
            Self::Num3 => Some('3'),
            Self::Num4 => Some('4'),
            Self::Num5 => Some('5'),
            Self::Num6 => Some('6'),
            Self::Num7 => Some('7'),
            Self::Num8 => Some('8'),
            Self::Num9 => Some('9'),
            Self::Space => Some(' '),
            _ => None,
        }
    }
}

/// Modifier flags matching CGEventFlags
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub option: bool,
    pub command: bool,
    pub caps_lock: bool,
}

impl Modifiers {
    pub fn from_cg_flags(flags: u64) -> Self {
        const SHIFT_MASK: u64 = 0x00020000;
        const CONTROL_MASK: u64 = 0x00040000;
        const OPTION_MASK: u64 = 0x00080000;
        const COMMAND_MASK: u64 = 0x00100000;
        const CAPS_LOCK_MASK: u64 = 0x00010000;

        Self {
            shift: flags & SHIFT_MASK != 0,
            control: flags & CONTROL_MASK != 0,
            option: flags & OPTION_MASK != 0,
            command: flags & COMMAND_MASK != 0,
            caps_lock: flags & CAPS_LOCK_MASK != 0,
        }
    }

    pub fn to_cg_flags(self) -> u64 {
        let mut flags = 0u64;
        if self.shift {
            flags |= 0x00020000;
        }
        if self.control {
            flags |= 0x00040000;
        }
        if self.option {
            flags |= 0x00080000;
        }
        if self.command {
            flags |= 0x00100000;
        }
        if self.caps_lock {
            flags |= 0x00010000;
        }
        flags
    }

}

/// A key event with code and modifiers
#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    pub code: u16,
    pub modifiers: Modifiers,
    pub is_key_down: bool,
}

impl KeyEvent {
    pub fn keycode(&self) -> Option<KeyCode> {
        KeyCode::from_raw(self.code)
    }
}
