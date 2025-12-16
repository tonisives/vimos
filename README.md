# OVIM

System-wide Vim keybindings for macOS.

OVIM is a lightweight menu bar application that brings Vim's modal editing to every app on your Mac. Press a key to toggle between Insert and Normal mode anywhere - in your browser, text editors, terminal, or any other application.

<!-- TODO: Add hero screenshot or GIF showing OVIM in action -->
<!-- ![OVIM Demo](screenshots/demo.gif) -->

## Features

- **System-wide Vim modes** - Normal, Insert, and Visual modes work in any macOS application
- **Menu bar application** - Runs quietly in the background, accessible from the menu bar
- **Mode indicator** - Small floating widget shows your current mode at a glance
- **Configurable activation key** - Default is Caps Lock, but you can customize it with modifier keys
- **Per-application ignore list** - Disable OVIM for specific apps that have their own Vim mode
- **Customizable widgets** - Display battery status, caps lock state, or selection info
- **Launch at login** - Optionally start OVIM when your Mac boots

## Installation

### Homebrew

```bash
brew install --cask ovim
```

### GitHub Releases

Download the latest `.dmg` from the [Releases](https://github.com/tonisives/ovim/releases) page and drag OVIM to your Applications folder.

### Build from Source

See [Building from Source](#building-from-source) below.

## Requirements

- macOS 10.15 (Catalina) or later
- **Accessibility permission** - OVIM needs permission to capture keyboard input. You'll be prompted to grant this on first launch in System Settings > Privacy & Security > Accessibility.

## Usage

### Getting Started

1. Launch OVIM - it will appear in your menu bar
2. Grant Accessibility permission when prompted
3. Press **Caps Lock** (or your configured activation key) to toggle between modes

### Modes

- **Insert mode** (default) - Keys work normally, passed through to the active application
- **Normal mode** - Vim navigation and commands are active
- **Visual mode** - Select text using Vim motions

### Mode Indicator

The floating indicator shows your current mode:
- Displayed near the top-left of your screen by default
- Position, opacity, and size are customizable in Settings

### Accessing Settings

Click the OVIM icon in the menu bar and select **Settings**.

## Supported Vim Commands

### Modes

| Key | Action |
|-----|--------|
| `Esc` | Return to Normal mode |
| `i` | Insert at cursor |
| `I` | Insert at line start |
| `a` | Append after cursor |
| `A` | Append at line end |
| `o` | Open line below |
| `O` | Open line above |
| `v` | Enter Visual mode |
| `s` | Substitute character (delete and insert) |
| `S` | Substitute line (delete line and insert) |

### Motions

| Key | Action |
|-----|--------|
| `h` | Move left |
| `j` | Move down |
| `k` | Move up |
| `l` | Move right |
| `w` | Word forward |
| `b` | Word backward |
| `e` | Word end |
| `ge` | Word end backward |
| `0` | Line start |
| `$` | Line end |
| `{` | Paragraph up |
| `}` | Paragraph down |
| `gg` | Document start |
| `G` | Document end |
| `Ctrl+u` | Half page up |
| `Ctrl+d` | Half page down |

### Operators

Operators can be combined with motions (e.g., `dw` deletes a word, `y$` yanks to line end).

| Key | Action |
|-----|--------|
| `d` | Delete |
| `y` | Yank (copy) |
| `c` | Change (delete and enter insert mode) |

### Text Objects

Use with operators (e.g., `diw` deletes inner word, `yaw` yanks around word).

| Key | Action |
|-----|--------|
| `iw` | Inner word |
| `aw` | Around word (includes surrounding space) |

### Commands

| Key | Action |
|-----|--------|
| `x` | Delete character under cursor |
| `X` | Delete character before cursor |
| `D` | Delete to line end |
| `C` | Change to line end |
| `Y` | Yank line |
| `dd` | Delete line |
| `yy` | Yank line |
| `cc` | Change line |
| `J` | Join lines |
| `p` | Paste after cursor |
| `P` | Paste before cursor |
| `u` | Undo |
| `Ctrl+r` | Redo |
| `>>` | Indent line |
| `<<` | Outdent line |

### Counts

Prefix motions and commands with a number to repeat them:
- `5j` - Move down 5 lines
- `3dw` - Delete 3 words
- `10x` - Delete 10 characters

## Settings

### General
- **Activation key** - Key to toggle Vim mode (default: Caps Lock)
- **Key modifiers** - Require Shift, Control, Option, or Command with the activation key
- **Launch at login** - Start OVIM automatically when you log in
- **Show in menu bar** - Toggle menu bar icon visibility

### Indicator
- **Position** - Where to display the mode indicator on screen
- **Opacity** - Transparency of the indicator (0-100%)
- **Size** - Scale of the indicator

### Widgets
- **Top widget** - Information displayed above the mode indicator
- **Bottom widget** - Information displayed below the mode indicator
- Available widgets: Battery, Caps Lock state, Selection info

### Ignored Apps
- Add applications where OVIM should be disabled
- Useful for apps with their own Vim mode (e.g., terminal emulators, VS Code with Vim extension)

<!-- TODO: Add screenshots -->
<!--
### Screenshots

#### Settings Window
![Settings](screenshots/settings.png)

#### Mode Indicator
![Indicator](screenshots/indicator.png)

#### Widgets
![Widgets](screenshots/widgets.png)
-->

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) toolchain
- [Node.js](https://nodejs.org/) (v18 or later recommended)
- [pnpm](https://pnpm.io/) package manager

### Build

```bash
# Clone the repository
git clone https://github.com/tonisives/ovim.git
cd ovim

# Install dependencies
pnpm install

# Build the app (universal binary for Intel + Apple Silicon)
pnpm tauri build

# The built app will be in src-tauri/target/release/bundle/
```

### Development

```bash
# Run in development mode with hot reload
pnpm tauri dev
```

## License

MIT License - see [LICENSE](LICENSE) for details.
