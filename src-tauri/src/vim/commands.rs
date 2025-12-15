use crate::keyboard;

/// Vim commands that can be executed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimCommand {
    // Basic motions
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,

    // Word motions
    WordForward,
    WordEnd,
    WordBackward,

    // Line motions
    LineStart,
    LineEnd,

    // Document motions
    DocumentStart,
    DocumentEnd,

    // Page motions
    PageUp,
    PageDown,
    HalfPageUp,
    HalfPageDown,

    // Insert mode transitions
    InsertAtCursor,
    InsertAtLineStart,
    AppendAfterCursor,
    AppendAtLineEnd,
    OpenLineBelow,
    OpenLineAbove,

    // Operations
    DeleteChar,
    DeleteCharBefore,
    DeleteLine,
    DeleteToLineEnd,
    YankLine,
    ChangeLine,
    ChangeToLineEnd,

    // Clipboard
    Paste,
    PasteBefore,

    // Undo/Redo
    Undo,
    Redo,

    // Mode transitions
    EnterNormalMode,
    EnterInsertMode,
    EnterVisualMode,
    ExitToInsertMode,
}

impl VimCommand {
    /// Execute the command, optionally with visual selection
    pub fn execute(&self, count: u32, select: bool) -> Result<(), String> {
        match self {
            // Basic motions
            Self::MoveLeft => keyboard::cursor_left(count, select),
            Self::MoveRight => keyboard::cursor_right(count, select),
            Self::MoveUp => keyboard::cursor_up(count, select),
            Self::MoveDown => keyboard::cursor_down(count, select),

            // Word motions
            Self::WordForward | Self::WordEnd => keyboard::word_forward(count, select),
            Self::WordBackward => keyboard::word_backward(count, select),

            // Line motions
            Self::LineStart => keyboard::line_start(select),
            Self::LineEnd => keyboard::line_end(select),

            // Document motions
            Self::DocumentStart => keyboard::document_start(select),
            Self::DocumentEnd => keyboard::document_end(select),

            // Page motions
            Self::PageUp | Self::HalfPageUp => keyboard::page_up(select),
            Self::PageDown | Self::HalfPageDown => keyboard::page_down(select),

            // Insert mode transitions
            Self::InsertAtCursor | Self::EnterInsertMode | Self::ExitToInsertMode => Ok(()),
            Self::InsertAtLineStart => keyboard::line_start(false),
            Self::AppendAfterCursor => keyboard::cursor_right(1, false),
            Self::AppendAtLineEnd => keyboard::line_end(false),
            Self::OpenLineBelow => keyboard::new_line_below(),
            Self::OpenLineAbove => keyboard::new_line_above(),

            // Operations
            Self::DeleteChar => {
                for _ in 0..count {
                    keyboard::delete_char()?;
                }
                Ok(())
            }
            Self::DeleteCharBefore => {
                for _ in 0..count {
                    keyboard::backspace()?;
                }
                Ok(())
            }
            Self::DeleteLine => {
                // Select entire line and delete
                keyboard::line_start(false)?;
                keyboard::line_end(true)?;
                keyboard::cut()
            }
            Self::DeleteToLineEnd => {
                keyboard::line_end(true)?;
                keyboard::cut()
            }
            Self::YankLine => {
                keyboard::line_start(false)?;
                keyboard::line_end(true)?;
                keyboard::copy()
            }
            Self::ChangeLine => {
                keyboard::line_start(false)?;
                keyboard::line_end(true)?;
                keyboard::cut()
            }
            Self::ChangeToLineEnd => {
                keyboard::line_end(true)?;
                keyboard::cut()
            }

            // Clipboard
            Self::Paste | Self::PasteBefore => keyboard::paste(),

            // Undo/Redo
            Self::Undo => keyboard::undo(),
            Self::Redo => keyboard::redo(),

            // Mode changes handled by state machine
            Self::EnterNormalMode | Self::EnterVisualMode => Ok(()),
        }
    }
}

/// Pending operator (d, y, c)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Yank,
    Change,
}

impl Operator {
    /// Execute operator with the given motion
    pub fn execute_with_motion(&self, motion: VimCommand, count: u32) -> Result<bool, String> {
        // First, select the text
        motion.execute(count, true)?;

        // Then apply the operator
        match self {
            Self::Delete => {
                keyboard::cut()?;
                Ok(false) // Stay in normal mode
            }
            Self::Yank => {
                keyboard::copy()?;
                // Move cursor back (yank doesn't delete)
                keyboard::cursor_left(1, false)?;
                Ok(false) // Stay in normal mode
            }
            Self::Change => {
                keyboard::cut()?;
                Ok(true) // Enter insert mode
            }
        }
    }
}
