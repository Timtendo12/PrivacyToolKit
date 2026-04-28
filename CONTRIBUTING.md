# Contributing to PrivacyToolKit

Thank you for your interest in contributing! This document covers everything you need to get started.

## Before You Start

- Check the [issue tracker](https://github.com/timtendo12/PrivacyToolKit/issues) to see if your idea or bug is already being discussed.
- For significant changes, open an issue first to discuss the approach before writing code.

## Development Setup

1. Install [Rust](https://rustup.rs) (stable toolchain).
2. Clone the repo and build:
   ```bash
   git clone https://github.com/timtendo12/PrivacyToolKit
   cd PrivacyToolKit
   cargo build
   ```

## Making Changes

- Keep pull requests focused — one feature or fix per PR.
- Run the following before pushing:
  ```bash
  cargo fmt       # format your code
  cargo clippy    # fix any lint warnings
  cargo test      # make sure tests pass
  ```
- Clippy warnings should be resolved, not suppressed with `#[allow(...)]` unless there is a clear reason.

## Adding a New Tool

Each tool lives in its own file under `src/pages/`. To add one:

1. Create `src/pages/your_tool.rs` with a state struct and a `pub fn show(ui: &mut egui::Ui, state: &mut YourToolState)`.
2. Register it in `src/pages/mod.rs`.
3. Add a variant to the `Page` enum in `src/app.rs`.
4. Add a nav button and route in `src/app.rs`.

## Commit Messages

Use short, descriptive messages in the imperative mood:

```
Add batch processing to metadata remover
Fix placeholder not rendering on first frame
```

## Pull Requests

- Target the `develop` branch.
- Fill in the PR description with what changed and why.
- Link any related issues.
