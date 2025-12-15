use tokio::sync::broadcast;

use crate::keyboard::{self, KeyCode, KeyEvent, Modifiers};
use super::commands::{Operator, VimCommand};
use super::modes::VimMode;

/// Result of processing a key event
#[derive(Debug, Clone)]
pub enum ProcessResult {
    /// Suppress the key event entirely
    Suppress,
    /// Pass the key event through unchanged
    PassThrough,
    /// Mode changed (emit event)
    ModeChanged(VimMode),
}

/// Vim state machine
pub struct VimState {
    mode: VimMode,
    /// The key that toggles vim mode (default: CapsLock)
    vim_key: KeyCode,
    /// Pending count for repeat (e.g., "5" in "5j")
    pending_count: Option<u32>,
    /// Pending operator (d, y, c)
    pending_operator: Option<Operator>,
    /// Pending g key for gg, gt, gT
    pending_g: bool,
    /// Channel to emit mode changes
    mode_tx: broadcast::Sender<VimMode>,
}

impl VimState {
    pub fn new() -> (Self, broadcast::Receiver<VimMode>) {
        let (mode_tx, mode_rx) = broadcast::channel(16);
        (
            Self {
                mode: VimMode::Insert,
                vim_key: KeyCode::CapsLock,
                pending_count: None,
                pending_operator: None,
                pending_g: false,
                mode_tx,
            },
            mode_rx,
        )
    }

    pub fn mode(&self) -> VimMode {
        self.mode
    }

    pub fn set_vim_key(&mut self, key: KeyCode) {
        self.vim_key = key;
    }

    fn set_mode(&mut self, mode: VimMode) {
        if self.mode != mode {
            self.mode = mode;
            self.reset_pending();
            let _ = self.mode_tx.send(mode);
        }
    }

    /// Set mode externally (from CLI/IPC)
    pub fn set_mode_external(&mut self, mode: VimMode) {
        self.set_mode(mode);
    }

    /// Toggle between insert and normal mode (for CLI/IPC)
    pub fn toggle_mode(&mut self) -> VimMode {
        let new_mode = match self.mode {
            VimMode::Insert => VimMode::Normal,
            VimMode::Normal | VimMode::Visual => VimMode::Insert,
        };
        self.set_mode(new_mode);
        new_mode
    }

    fn reset_pending(&mut self) {
        self.pending_count = None;
        self.pending_operator = None;
        self.pending_g = false;
    }

    fn get_count(&self) -> u32 {
        self.pending_count.unwrap_or(1)
    }

    /// Process a key event and return what to do with it
    pub fn process_key(&mut self, event: KeyEvent) -> ProcessResult {
        // For key up events in Normal/Visual mode, suppress keys that we would suppress on key down
        // This prevents the character from being typed
        if !event.is_key_down {
            // In Insert mode, pass through all key up events
            if self.mode == VimMode::Insert {
                return ProcessResult::PassThrough;
            }

            // In Normal/Visual mode, suppress key up for keys we handle
            let keycode = match event.keycode() {
                Some(k) => k,
                None => return ProcessResult::PassThrough,
            };

            // Check if this is a key we would handle (suppress its key up too)
            let should_suppress = matches!(
                keycode,
                KeyCode::H | KeyCode::J | KeyCode::K | KeyCode::L |
                KeyCode::W | KeyCode::E | KeyCode::B |
                KeyCode::Num0 | KeyCode::G |
                KeyCode::D | KeyCode::Y | KeyCode::C | KeyCode::X |
                KeyCode::I | KeyCode::A | KeyCode::O |
                KeyCode::V | KeyCode::P | KeyCode::U
            );

            return if should_suppress {
                ProcessResult::Suppress
            } else {
                ProcessResult::PassThrough
            };
        }

        let keycode = match event.keycode() {
            Some(k) => k,
            None => return ProcessResult::PassThrough,
        };

        // Check for vim key toggle
        if keycode == self.vim_key {
            return self.handle_vim_key();
        }

        match self.mode {
            VimMode::Insert => ProcessResult::PassThrough,
            VimMode::Normal => self.process_normal_mode(keycode, &event.modifiers),
            VimMode::Visual => self.process_visual_mode(keycode, &event.modifiers),
        }
    }

    fn handle_vim_key(&mut self) -> ProcessResult {
        match self.mode {
            VimMode::Insert => {
                self.set_mode(VimMode::Normal);
                ProcessResult::ModeChanged(VimMode::Normal)
            }
            VimMode::Normal | VimMode::Visual => {
                self.set_mode(VimMode::Insert);
                ProcessResult::ModeChanged(VimMode::Insert)
            }
        }
    }

    fn process_normal_mode(&mut self, keycode: KeyCode, modifiers: &Modifiers) -> ProcessResult {
        // Escape always goes to insert mode
        if keycode == KeyCode::Escape {
            self.set_mode(VimMode::Insert);
            return ProcessResult::ModeChanged(VimMode::Insert);
        }

        // Handle pending g
        if self.pending_g {
            self.pending_g = false;
            return self.handle_g_combo(keycode);
        }

        // Handle count accumulation (1-9, then 0-9)
        if let Some(digit) = self.keycode_to_digit(keycode) {
            if digit != 0 || self.pending_count.is_some() {
                let current = self.pending_count.unwrap_or(0);
                self.pending_count = Some(current * 10 + digit);
                return ProcessResult::Suppress;
            }
        }

        // Handle operators
        if self.pending_operator.is_some() {
            return self.handle_operator_motion(keycode, modifiers);
        }

        // Check for control key combinations
        if modifiers.control {
            return self.handle_control_combo(keycode);
        }

        // Normal mode commands
        self.handle_normal_command(keycode, modifiers)
    }

    fn process_visual_mode(&mut self, keycode: KeyCode, _modifiers: &Modifiers) -> ProcessResult {
        // Escape exits visual mode
        if keycode == KeyCode::Escape {
            self.set_mode(VimMode::Normal);
            return ProcessResult::ModeChanged(VimMode::Normal);
        }

        // v toggles back to normal
        if keycode == KeyCode::V {
            self.set_mode(VimMode::Normal);
            return ProcessResult::ModeChanged(VimMode::Normal);
        }

        // Handle motions (with selection)
        let count = self.get_count();
        self.pending_count = None;

        let result = match keycode {
            KeyCode::H => VimCommand::MoveLeft.execute(count, true),
            KeyCode::J => VimCommand::MoveDown.execute(count, true),
            KeyCode::K => VimCommand::MoveUp.execute(count, true),
            KeyCode::L => VimCommand::MoveRight.execute(count, true),
            KeyCode::W | KeyCode::E => VimCommand::WordForward.execute(count, true),
            KeyCode::B => VimCommand::WordBackward.execute(count, true),
            KeyCode::Num0 => VimCommand::LineStart.execute(1, true),

            // Operations
            KeyCode::D | KeyCode::X => {
                if keyboard::cut().is_ok() {
                    self.set_mode(VimMode::Normal);
                    return ProcessResult::ModeChanged(VimMode::Normal);
                }
                return ProcessResult::Suppress;
            }
            KeyCode::Y => {
                if keyboard::copy().is_ok() {
                    self.set_mode(VimMode::Normal);
                    return ProcessResult::ModeChanged(VimMode::Normal);
                }
                return ProcessResult::Suppress;
            }
            KeyCode::C => {
                if keyboard::cut().is_ok() {
                    self.set_mode(VimMode::Insert);
                    return ProcessResult::ModeChanged(VimMode::Insert);
                }
                return ProcessResult::Suppress;
            }

            _ => return ProcessResult::PassThrough,
        };

        if result.is_ok() {
            ProcessResult::Suppress
        } else {
            ProcessResult::PassThrough
        }
    }

    fn handle_normal_command(&mut self, keycode: KeyCode, modifiers: &Modifiers) -> ProcessResult {
        let count = self.get_count();
        self.pending_count = None;

        let result = match keycode {
            // Basic motions
            KeyCode::H => VimCommand::MoveLeft.execute(count, false),
            KeyCode::J => VimCommand::MoveDown.execute(count, false),
            KeyCode::K => VimCommand::MoveUp.execute(count, false),
            KeyCode::L => VimCommand::MoveRight.execute(count, false),

            // Word motions
            KeyCode::W => VimCommand::WordForward.execute(count, false),
            KeyCode::E => VimCommand::WordEnd.execute(count, false),
            KeyCode::B => VimCommand::WordBackward.execute(count, false),

            // Line motions
            KeyCode::Num0 => VimCommand::LineStart.execute(1, false),

            // g commands
            KeyCode::G => {
                if modifiers.shift {
                    // G = go to end
                    VimCommand::DocumentEnd.execute(1, false)
                } else {
                    // g = start g combo
                    self.pending_g = true;
                    return ProcessResult::Suppress;
                }
            }

            // Operators
            KeyCode::D => {
                if self.pending_operator == Some(Operator::Delete) {
                    // dd = delete line
                    self.pending_operator = None;
                    VimCommand::DeleteLine.execute(count, false)
                } else {
                    self.pending_operator = Some(Operator::Delete);
                    return ProcessResult::Suppress;
                }
            }
            KeyCode::Y => {
                if self.pending_operator == Some(Operator::Yank) {
                    // yy = yank line
                    self.pending_operator = None;
                    VimCommand::YankLine.execute(count, false)
                } else {
                    self.pending_operator = Some(Operator::Yank);
                    return ProcessResult::Suppress;
                }
            }
            KeyCode::C => {
                if self.pending_operator == Some(Operator::Change) {
                    // cc = change line
                    self.pending_operator = None;
                    if VimCommand::ChangeLine.execute(count, false).is_ok() {
                        self.set_mode(VimMode::Insert);
                        return ProcessResult::ModeChanged(VimMode::Insert);
                    }
                    return ProcessResult::Suppress;
                } else {
                    self.pending_operator = Some(Operator::Change);
                    return ProcessResult::Suppress;
                }
            }

            // Single-key operations
            KeyCode::X => VimCommand::DeleteChar.execute(count, false),

            // Insert mode entries
            KeyCode::I => {
                if modifiers.shift {
                    // I = insert at line start
                    let _ = VimCommand::InsertAtLineStart.execute(1, false);
                }
                self.set_mode(VimMode::Insert);
                return ProcessResult::ModeChanged(VimMode::Insert);
            }
            KeyCode::A => {
                if modifiers.shift {
                    // A = append at line end
                    let _ = VimCommand::AppendAtLineEnd.execute(1, false);
                } else {
                    // a = append after cursor
                    let _ = VimCommand::AppendAfterCursor.execute(1, false);
                }
                self.set_mode(VimMode::Insert);
                return ProcessResult::ModeChanged(VimMode::Insert);
            }
            KeyCode::O => {
                if modifiers.shift {
                    let _ = VimCommand::OpenLineAbove.execute(1, false);
                } else {
                    let _ = VimCommand::OpenLineBelow.execute(1, false);
                }
                self.set_mode(VimMode::Insert);
                return ProcessResult::ModeChanged(VimMode::Insert);
            }

            // Visual mode
            KeyCode::V => {
                self.set_mode(VimMode::Visual);
                return ProcessResult::ModeChanged(VimMode::Visual);
            }

            // Clipboard
            KeyCode::P => {
                if modifiers.shift {
                    VimCommand::PasteBefore.execute(count, false)
                } else {
                    VimCommand::Paste.execute(count, false)
                }
            }

            // Undo/Redo
            KeyCode::U => VimCommand::Undo.execute(count, false),

            _ => return ProcessResult::PassThrough,
        };

        if result.is_ok() {
            ProcessResult::Suppress
        } else {
            ProcessResult::PassThrough
        }
    }

    fn handle_control_combo(&mut self, keycode: KeyCode) -> ProcessResult {
        let count = self.get_count();
        self.pending_count = None;

        let result = match keycode {
            KeyCode::F => VimCommand::PageDown.execute(count, false),
            KeyCode::B => VimCommand::PageUp.execute(count, false),
            KeyCode::D => VimCommand::HalfPageDown.execute(count, false),
            KeyCode::U => VimCommand::HalfPageUp.execute(count, false),
            KeyCode::R => VimCommand::Redo.execute(count, false),
            _ => return ProcessResult::PassThrough,
        };

        if result.is_ok() {
            ProcessResult::Suppress
        } else {
            ProcessResult::PassThrough
        }
    }

    fn handle_g_combo(&mut self, keycode: KeyCode) -> ProcessResult {
        let result = match keycode {
            KeyCode::G => {
                // gg = go to start
                VimCommand::DocumentStart.execute(1, false)
            }
            _ => return ProcessResult::PassThrough,
        };

        if result.is_ok() {
            ProcessResult::Suppress
        } else {
            ProcessResult::PassThrough
        }
    }

    fn handle_operator_motion(&mut self, keycode: KeyCode, modifiers: &Modifiers) -> ProcessResult {
        let operator = match self.pending_operator.take() {
            Some(op) => op,
            None => return ProcessResult::PassThrough,
        };

        let count = self.get_count();
        self.pending_count = None;

        // Map keycode to motion
        let motion = match keycode {
            KeyCode::H => Some(VimCommand::MoveLeft),
            KeyCode::J => Some(VimCommand::MoveDown),
            KeyCode::K => Some(VimCommand::MoveUp),
            KeyCode::L => Some(VimCommand::MoveRight),
            KeyCode::W => Some(VimCommand::WordForward),
            KeyCode::E => Some(VimCommand::WordEnd),
            KeyCode::B => Some(VimCommand::WordBackward),
            KeyCode::Num0 => Some(VimCommand::LineStart),
            KeyCode::G if modifiers.shift => Some(VimCommand::DocumentEnd),
            _ => None,
        };

        if let Some(motion) = motion {
            match operator.execute_with_motion(motion, count) {
                Ok(enter_insert) => {
                    if enter_insert {
                        self.set_mode(VimMode::Insert);
                        return ProcessResult::ModeChanged(VimMode::Insert);
                    }
                    ProcessResult::Suppress
                }
                Err(_) => ProcessResult::PassThrough,
            }
        } else {
            // Invalid motion, reset
            self.reset_pending();
            ProcessResult::Suppress
        }
    }

    fn keycode_to_digit(&self, keycode: KeyCode) -> Option<u32> {
        match keycode {
            KeyCode::Num0 => Some(0),
            KeyCode::Num1 => Some(1),
            KeyCode::Num2 => Some(2),
            KeyCode::Num3 => Some(3),
            KeyCode::Num4 => Some(4),
            KeyCode::Num5 => Some(5),
            KeyCode::Num6 => Some(6),
            KeyCode::Num7 => Some(7),
            KeyCode::Num8 => Some(8),
            KeyCode::Num9 => Some(9),
            _ => None,
        }
    }
}

impl Default for VimState {
    fn default() -> Self {
        Self::new().0
    }
}
