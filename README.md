# Hex Map Editor

![Project Icon](./icon.png)

An easy to use cross-platform hexmap editor.
I have found drawing out maps by hand for my games to be tedious, and that other tools don't offer the approach I need.

## Features
- Tools
  - [X] Draw / Erase tools
  - [ ] Resizeable tools
  - [ ] Paint bucket tool
- Layers
  - [X] Hide / Rename / Remove
  - [X] Re-colourable with pleasing defaults
  - [X] Partial transparency
  - [ ] Proc-gen. layers 
- Data management
  - [X] Export to PNG
  - [ ] Export to SVG & PDF
  - [ ] Map saving
  - [ ] Incremental saves



## Build Instructions

- Linux / Windows:  `cargo build --release`

- Web:  `trunk build --release`

  Depends on the [Rust](https://rust-lang.org/) toolchain, the `wasm32-unknown-unknown` target, and [Trunk](https://trunk-rs.github.io/trunk/) to be installed.
