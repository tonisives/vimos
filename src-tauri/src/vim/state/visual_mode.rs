use crate::keyboard::{KeyCode, Modifiers};
use super::super::commands::VimCommand;
use super::super::modes::VimMode;
use super::action::VimAction;
use super::{ProcessResult, TextObjectModifier};
use super::VimState;

impl VimState {
    pub(super) fn process_visual_mode_with_modifiers(&mut self, keycode: KeyCode, modifiers: &Modifiers) -> ProcessResult {
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

        // Handle pending g
        if self.pending_g {
            self.pending_g = false;
            return self.handle_visual_g_combo(keycode);
        }

        // Handle pending text object modifier
        if let Some(modifier) = self.pending_text_object.take() {
            return self.handle_visual_text_object(keycode, modifier);
        }

        // Handle count accumulation (1-9, then 0-9)
        // Must check this BEFORE processing other keys
        // Only accumulate if shift is NOT pressed (shift+number = special chars like $ ^)
        if !modifiers.shift {
            if let Some(digit) = keycode.to_digit() {
                if digit != 0 || self.pending_count.is_some() {
                    let current = self.pending_count.unwrap_or(0);
                    self.pending_count = Some(current * 10 + digit);
                    return ProcessResult::Suppress;
                }
            }
        }

        let count = self.get_count();
        self.pending_count = None;

        match keycode {
            // Basic motions (with selection)
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

            // Word motions
            KeyCode::W | KeyCode::E => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::WordForward, count, select: true
            }),
            KeyCode::B => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::WordBackward, count, select: true
            }),

            // Line motions
            KeyCode::Num0 => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::LineStart, count: 1, select: true
            }),
            // $ = line end
            KeyCode::Num4 if modifiers.shift => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::LineEnd, count: 1, select: true
            }),
            // ^ = line start
            KeyCode::Num6 if modifiers.shift => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::LineStart, count: 1, select: true
            }),

            // Paragraph motions
            KeyCode::LeftBracket if modifiers.shift => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::ParagraphUp, count, select: true
            }),
            KeyCode::RightBracket if modifiers.shift => ProcessResult::SuppressWithAction(VimAction::Command {
                command: VimCommand::ParagraphDown, count, select: true
            }),

            // Document motions
            KeyCode::G => {
                if modifiers.shift {
                    // G = document end
                    ProcessResult::SuppressWithAction(VimAction::Command {
                        command: VimCommand::DocumentEnd, count: 1, select: true
                    })
                } else {
                    // g = start g combo
                    self.pending_g = true;
                    ProcessResult::Suppress
                }
            }

            // Text object modifiers
            KeyCode::I if !modifiers.shift => {
                self.pending_text_object = Some(TextObjectModifier::Inner);
                ProcessResult::Suppress
            }
            KeyCode::A if !modifiers.shift => {
                self.pending_text_object = Some(TextObjectModifier::Around);
                ProcessResult::Suppress
            }

            // Operations on selection
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

    fn handle_visual_g_combo(&mut self, keycode: KeyCode) -> ProcessResult {
        let count = self.get_count();
        self.pending_count = None;

        match keycode {
            KeyCode::G => {
                // gg = document start with selection
                ProcessResult::SuppressWithAction(VimAction::Command {
                    command: VimCommand::DocumentStart, count: 1, select: true
                })
            }
            KeyCode::E => {
                // ge = end of previous word with selection
                ProcessResult::SuppressWithAction(VimAction::Command {
                    command: VimCommand::WordEndBackward, count, select: true
                })
            }
            _ => ProcessResult::PassThrough,
        }
    }

    fn handle_visual_text_object(&self, keycode: KeyCode, modifier: TextObjectModifier) -> ProcessResult {
        // In visual mode, text objects extend the selection
        if keycode == KeyCode::W {
            let text_object = match modifier {
                TextObjectModifier::Inner => VimCommand::InnerWord,
                TextObjectModifier::Around => VimCommand::AroundWord,
            };
            // Execute the text object to extend selection
            ProcessResult::SuppressWithAction(VimAction::Command {
                command: text_object, count: 1, select: false
            })
        } else {
            ProcessResult::PassThrough
        }
    }
}
