use crate::keyboard::{self, KeyCode};
use super::super::commands::{Operator, VimCommand};

/// Action to execute after suppressing the key event
#[derive(Debug, Clone)]
pub enum VimAction {
    /// Execute a vim command
    Command { command: VimCommand, count: u32, select: bool },
    /// Execute an operator with a motion
    OperatorMotion { operator: Operator, motion: VimCommand, count: u32 },
    /// Execute an operator with a text object
    TextObject { operator: Operator, text_object: VimCommand, count: u32 },
    /// Replace character at cursor
    ReplaceChar { keycode: KeyCode, shift: bool, count: u32 },
    /// Cut (Cmd+X)
    Cut,
    /// Copy (Cmd+C)
    Copy,
}

impl VimAction {
    /// Execute the action
    pub fn execute(&self) -> Result<bool, String> {
        match self {
            VimAction::Command { command, count, select } => {
                command.execute(*count, *select)?;
                Ok(false)
            }
            VimAction::OperatorMotion { operator, motion, count } => {
                operator.execute_with_motion(*motion, *count)
            }
            VimAction::TextObject { operator, text_object, count } => {
                // Execute the text object selection
                for _ in 0..*count {
                    text_object.execute(1, false)?;
                }
                // Apply the operator
                match operator {
                    Operator::Delete => {
                        keyboard::cut()?;
                        Ok(false)
                    }
                    Operator::Yank => {
                        keyboard::copy()?;
                        keyboard::cursor_left(1, false)?;
                        Ok(false)
                    }
                    Operator::Change => {
                        keyboard::cut()?;
                        Ok(true) // Enter insert mode
                    }
                }
            }
            VimAction::ReplaceChar { keycode, shift, count } => {
                // Delete char(s) and type replacement
                for _ in 0..*count {
                    keyboard::delete_char()?;
                    keyboard::type_char(*keycode, *shift)?;
                }
                Ok(false)
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
