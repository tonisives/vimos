# Vim Keybindings

## Mode Switching

| Key | Action |
| --- | ------ |
| `Esc` | Return to Normal mode |
| `i` / `I` | Insert at cursor / line start |
| `a` / `A` | Append after cursor / line end |
| `o` / `O` | Open line below / above |
| `v` | Enter Visual mode |
| `s` / `S` | Substitute character / line |

## Motions

| Key | Action |
| --- | ------ |
| `h` `j` `k` `l` | Left, down, up, right |
| `w` / `b` / `e` | Word forward / backward / end |
| `0` / `$` | Line start / end |
| `{` / `}` | Paragraph up / down |
| `gg` / `G` | Document start / end |
| `Ctrl+u` / `Ctrl+d` | Half page up / down |

## Operators + Text Objects

Operators combine with motions (e.g., `dw` deletes word, `y$` yanks to line end).

| Operator | Action |
| -------- | ------ |
| `d` | Delete |
| `y` | Yank (copy) |
| `c` | Change (delete + insert) |

| Text Object | Action |
| ----------- | ------ |
| `iw` / `aw` | Inner word / around word |

## Commands

| Key | Action |
| --- | ------ |
| `x` / `X` | Delete char under / before cursor |
| `D` / `C` / `Y` | Delete / change / yank to line end |
| `dd` / `yy` / `cc` | Delete / yank / change line |
| `J` | Join lines |
| `p` / `P` | Paste after / before cursor |
| `u` / `Ctrl+r` | Undo / redo |
| `>>` / `<<` | Indent / outdent line |

## Counts

Prefix with numbers: `5j` (move down 5), `3dw` (delete 3 words), `10x` (delete 10 chars).
