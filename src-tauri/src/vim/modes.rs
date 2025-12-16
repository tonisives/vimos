use serde::{Deserialize, Serialize};

/// The three vim modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum VimMode {
    /// Normal typing mode
    #[default]
    Insert,
    /// Vim command mode (hjkl navigation, operators)
    Normal,
    /// Visual selection mode
    Visual,
}

impl VimMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Insert => "insert",
            Self::Normal => "normal",
            Self::Visual => "visual",
        }
    }
}

impl std::fmt::Display for VimMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
