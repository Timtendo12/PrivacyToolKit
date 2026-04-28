# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build          # debug build
cargo build --release  # optimized build
cargo run            # build and run
cargo test           # run all tests
cargo test <name>    # run a single test by name
cargo clippy         # lint
cargo fmt            # format code
```

## Architecture

This is a Rust desktop GUI application using [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) (the egui application framework). The entry point is `src/main.rs`.

**eframe pattern**: The typical structure is to define an `App` struct that implements `eframe::App`, with `update()` as the main render/event loop called every frame. State lives on the `App` struct; UI is built imperatively inside `update()` using `egui` widgets.

The project is currently in its initial scaffolding stage.