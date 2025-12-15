use tokio::sync::broadcast;

use crate::keyboard::{self, KeyCode, KeyEvent, Modifiers};
use super::commands::{Operator, VimCommand};
use super::modes::VimMode;

/// Action to execute after suppressing the key event
#[derive(Debug, Clone)]
pub enum VimAction {
    /// No action needed
    None,
    /// Execute a vim command
    Command { command: VimCommand, count: u32, select: bool },
    /// Execute an operator with a motion
    OperatorMotion { operator: Operator, motion: VimCommand, count: u32 },
    /// Cut (Cmd+X)
    Cut,
    /// Copy (Cmd+C)
    Copy,
}

impl VimAction {
    /// Execute the action
    pub fn execute(&self) -> Result<bool, String> {
        match self {
            VimAction::None => Ok(false),
            VimAction::Command { command, count, select } => {
                command.execute(*count, *select)?;
                Ok(false)
            }
            VimAction::OperatorMotion { operator, motion, count } => {
                operator.execute_with_motion(*motion, *count)
            }
            VimAction::Cut => {
                keyboard::cut()?;
                Ok(false)
            }
            VimAction::Copy => {
                keyboard::copy()?;
                Ok(false)
            }
        }
    }
}

/// Result of processing a key event
#[derive(Debug, Clone)]
pub enum ProcessResult {
    /// Suppress the key event entirely (no action needed)
    Suppress,
    /// Suppress and execute an action (deferred execution)
    SuppressWithAction(VimAction),
    /// Pass the key event through unchanged
    PassThrough,
    /// Mode changed (emit event), with optional action to execute
    ModeChanged(VimMode, Option<VimAction>),
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
                ProcessResult::ModeChanged(VimMode::Normal, None)
            }
            VimMode::Normal | VimMode::Visual => {
                self.set_mode(VimMode::Insert);
                ProcessResult::ModeChanged(VimMode::Insert, None)
            }
        }
    }

    fn process_normal_mode(&mut self, keycode: KeyCode, modifiers: &Modifiers) -> ProcessResult {
        // Escape always goes to insert mode
        if keycode == KeyCode::Escape {
            self.set_mode(VimMode::Insert);
            return ProcessResult::ModeChanged(VimMode::Insert, None);
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
            return ProcessResult::ModeChanged(VimMode::Normal, None);
        }

        // v toggles back to normal
        if keycode == KeyCode::V {
            self.set_mode(VimMode::Normal);
            return ProcessResult::ModeChanged(VimMode::Normal, None);
        }

        // Handle motions (with selection)
        let count = self.get_count();
        self.pending_count = None;

        match keycode {
            KeyCode::H => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::MoveLeft, count, select: true
            }),
            KeyCode::J => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::MoveDown, count, select: true
            }),
            KeyCode::K => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::MoveUp, count, select: true
            }),
            KeyCode::L => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::MoveRight, count, select: true
            }),
            KeyCode::W | KeyCode::E => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::WordForward, count, select: true
            }),
            KeyCode::B => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::WordBackward, count, select: true
            }),
            KeyCode::Num0 => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::LineStart, count: 1, select: true
            }),

            // Operations
            KeyCode::D | KeyCode::X => {
                self.set_mode(VimMode::Normal);
                ProcessResult::ModeChanged(VimMode::Normal, Some(VimAction::Cut))
            }
            KeyCode::Y => {
                self.set_mode(VimMode::Normal);
                ProcessResult::ModeChanged(VimMode::Normal, Some(VimAction::Copy))
            }
            KeyCode::C => {
                self.set_mode(VimMode::Insert);
                ProcessResult::ModeChanged(VimMode::Insert, Some(VimAction::Cut))
            }

            _ => ProcessResult::PassThrough,
        }
    }

    fn handle_normal_command(&mut self, keycode: KeyCode, modifiers: &Modifiers) -> ProcessResult {
        let count = self.get_count();
        self.pending_count = None;

        match keycode {
            // Basic motions
            KeyCode::H => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::MoveLeft, count, select: false
            }),
            KeyCode::J => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::MoveDown, count, select: false
            }),
            KeyCode::K => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::MoveUp, count, select: false
            }),
            KeyCode::L => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::MoveRight, count, select: false
            }),

            // Word motions
            KeyCode::W => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::WordForward, count, select: false
            }),
            KeyCode::E => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::WordEnd, count, select: false
            }),
            KeyCode::B => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::WordBackward, count, select: false
            }),

            // Line motions
            KeyCode::Num0 => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::LineStart, count: 1, select: false
            }),

            // g commands
            KeyCode::G => {
                if modifiers.shift {
                    // G = go to end
                    ProcessResult::SuppressWithAction(VimAction::Command {
                        command: VimCommand::DocumentEnd, count: 1, select: false
                    })
                } else {
                    // g = start g combo
                    self.pending_g = true;
                    ProcessResult::Suppress
                }
            }

            // Operators
            KeyCode::D => {
                if self.pending_operator == Some(Operator::Delete) {
                    // dd = delete line
                    self.pending_operator = None;
                    ProcessResult::SuppressWithAction(VimAction::Command {
                        command: VimCommand::DeleteLine, count, select: false
                    })
                } else {
                    self.pending_operator = Some(Operator::Delete);
                    ProcessResult::Suppress
                }
            }
            KeyCode::Y => {
                if self.pending_operator == Some(Operator::Yank) {
                    // yy = yank line
                    self.pending_operator = None;
                    ProcessResult::SuppressWithAction(VimAction::Command {
                        command: VimCommand::YankLine, count, select: false
                    })
                } else {
                    self.pending_operator = Some(Operator::Yank);
                    ProcessResult::Suppress
                }
            }
            KeyCode::C => {
                if self.pending_operator == Some(Operator::Change) {
                    // cc = change line
                    self.pending_operator = None;
                    self.set_mode(VimMode::Insert);
                    ProcessResult::ModeChanged(VimMode::Insert, Some(VimAction::Command {
                        command: VimCommand::ChangeLine, count, select: false
                    }))
                } else {
                    self.pending_operator = Some(Operator::Change);
                    ProcessResult::Suppress
                }
            }

            // Single-key operations
            KeyCode::X => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::DeleteChar, count, select: false
            }),

            // Insert mode entries
            KeyCode::I => {
                self.set_mode(VimMode::Insert);
                if modifiers.shift {
                    // I = insert at line start
                    ProcessResult::ModeChanged(VimMode::Insert, Some(VimAction::Command {
                        command: VimCommand::InsertAtLineStart, count: 1, select: false
                    }))
                } else {
                    ProcessResult::ModeChanged(VimMode::Insert, None)
                }
            }
            KeyCode::A => {
                self.set_mode(VimMode::Insert);
                if modifiers.shift {
                    // A = append at line end
                    ProcessResult::ModeChanged(VimMode::Insert, Some(VimAction::Command {
                        command: VimCommand::AppendAtLineEnd, count: 1, select: false
                    }))
                } else {
                    // a = append after cursor
                    ProcessResult::ModeChanged(VimMode::Insert, Some(VimAction::Command {
                        command: VimCommand::AppendAfterCursor, count: 1, select: false
                    }))
                }
            }
            KeyCode::O => {
                self.set_mode(VimMode::Insert);
                if modifiers.shift {
                    ProcessResult::ModeChanged(VimMode::Insert, Some(VimAction::Command {
                        command: VimCommand::OpenLineAbove, count: 1, select: false
                    }))
                } else {
                    ProcessResult::ModeChanged(VimMode::Insert, Some(VimAction::Command {
                        command: VimCommand::OpenLineBelow, count: 1, select: false
                    }))
                }
            }

            // Visual mode
            KeyCode::V => {
                self.set_mode(VimMode::Visual);
                ProcessResult::ModeChanged(VimMode::Visual, None)
            }

            // Clipboard
            KeyCode::P => {
                let command = if modifiers.shift {
                    VimCommand::PasteBefore
                } else {
                    VimCommand::Paste
                };
                ProcessResult::SuppressWithAction(VimAction::Command {
                    command, count, select: false
                })
            }

            // Undo/Redo
            KeyCode::U => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::Undo, count, select: false
            }),

            _ => ProcessResult::PassThrough,
        }
    }

    fn handle_control_combo(&mut self, keycode: KeyCode) -> ProcessResult {
        let count = self.get_count();
        self.pending_count = None;

        let command = match keycode {
            KeyCode::F => VimCommand::PageDown,
            KeyCode::B => VimCommand::PageUp,
            KeyCode::D => VimCommand::HalfPageDown,
            KeyCode::U => VimCommand::HalfPageUp,
            KeyCode::R => VimCommand::Redo,
            _ => return ProcessResult::PassThrough,
        };

        ProcessResult::SuppressWithAction(VimAction::Command {
            command, count, select: false
        })
    }

    fn handle_g_combo(&mut self, keycode: KeyCode) -> ProcessResult {
        match keycode {
            KeyCode::G => {
                // gg = go to start
                ProcessResult::SuppressWithAction(VimAction::Command {
                    command: VimCommand::DocumentStart, count: 1, select: false
                })
            }
            _ => ProcessResult::PassThrough,
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
            // For change operator, we need to enter insert mode after the action
            if operator == Operator::Change {
                self.set_mode(VimMode::Insert);
                ProcessResult::ModeChanged(VimMode::Insert, Some(VimAction::OperatorMotion {
                    operator, motion, count
                }))
            } else {
                ProcessResult::SuppressWithAction(VimAction::OperatorMotion {
                    operator, motion, count
                })
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
