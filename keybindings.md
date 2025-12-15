Motions

- $ - Move to end of line (Shift+4)
- ^ - Move to first non-blank (Shift+6, same as 0 on macOS)
- { / } - Paragraph up/down (Option+Up/Down)
- ge - End of previous word

Shortcuts

- X - Delete char before cursor (backspace)
- D - Delete to end of line
- C - Change to end of line
- Y - Yank line (same as yy)
- s - Substitute char (delete + insert mode)
- S - Substitute line (same as cc)
- J - Join lines
- r{char} - Replace character at cursor

Text Objects

- diw / yiw / ciw - Inner word operations
- daw / yaw / caw - Around word operations
- viw / vaw - Visual mode word selection

Extended Operator Motions

All operators (d, y, c) now work with:

- $, ^ (line boundaries)
- {, } (paragraph)
- gg, G (document)

g-Prefix Commands

- gg - Document start (existing)
- ge - Previous word end
- gj / gk - Visual line movement (same as j/k)
- g0 / g$ - Screen line start/end

Indent/Outdent

- > > - Indent line (Tab)
- << - Outdent line (Shift+Tab)

Visual Mode

- All new motions work with selection
- Text object selection (viw, vaw)
- Count support for motions

Files Modified

- src-tauri/src/keyboard/inject.rs - New injection helpers
- src-tauri/src/keyboard/keycode.rs - Added to_char() method
- src-tauri/src/vim/commands.rs - New command variants
- src-tauri/src/vim/state/mod.rs - New state fields
- src-tauri/src/vim/state/normal_mode.rs - All new command handling
- src-tauri/src/vim/state/visual_mode.rs - Extended visual mode
- src-tauri/src/vim/state/action.rs - New action types
