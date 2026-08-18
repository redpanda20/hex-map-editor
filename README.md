# Hex Map Editor

![Project Icon](./icon.png)

An easy-to-use, cross-platform hex map editor built for tabletop and game design.
I have found drawing out maps by hand for my games to be tedious, and that other tools don't offer the approach I need.

Try it out now for free: [stephens.ac/hex-map-editor/](stephens.ac/hex-map-editor/)

## Features
- Tools
  - [X] Draw / erase tools for fast, precise map edits
  - [X] Paint bucket for filling large regions in one click
  - [ ] Resizable tools for finer control over brush scale
- Layers
  - [X] Create, remove, and reorder layers to keep complex maps organized
  - [X] Toggle visibility and rename layers on the fly
  - [X] Recolour layers instantly, with sensible defaults out of the box
  - [X] Partial transparency for layering terrain, overlays, and effects
- Data management
  - [X] Save and load scenes to pick up projects where you left off
  - [X] Export to PNG for easy sharing
  - [ ] Export to SVG & PDF for scalable, print-ready maps
  - [ ] Incremental saves to protect your work automatically

## Coming Soon
- **Image layers.** Drop in reference art, tokens, or custom assets as their own movable, transparent layer
- **SVG export.** Scalable, lightweight exports. A stepping stone to print-ready PDFs, and a faster PNG pipeline
- **Undo/Redo.** Full edit history so nothing is ever a mistake you can't take back
- **Keybinds.** Speed up your workflow with keyboard shortcuts for common actions

## Build Instructions

- Linux / Windows:  `cargo build --release`

- Web:  `trunk build --release`

  Depends on the [Rust](https://rust-lang.org/) toolchain, the `wasm32-unknown-unknown` target, and [Trunk](https://trunk-rs.github.io/trunk/) to be installed.
