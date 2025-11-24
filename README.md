## Dependencies

1. rust
1. A wlroots compatible compositor
1. Probably some wayland related header packages that I forgot about

## Install

1. Clone the repo
1. `cargo build --release`
1. Sym-link the executable at `./target/release/hyprland-window-rule` to some directory in your `$PATH` (e.g. `~/bin/)

## Usage

### Prerequisite

1. Add `source = window-rules/*` somewhere in your hyprland config file so that the rules you generate actually get used

This is very unfinished so right now you can only generate a rule to make a window float based on its initial title.

1. `hyprland-window-rule generate --float  --select-by title  --select-by initial-class`
1. Select a window with you mouse cursor

## Know Issues

The window selection boxes appear lower than they should because the desktop status bar's effect on this program's surface positions isn't taken into account when calculating where to draw stuff.
