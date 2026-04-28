# PrivacyToolKit

A lightweight desktop app for preserving your privacy. Built with Rust and [egui](https://github.com/emilk/egui).

> **Early development** — features are limited and the app is not yet stable.

## Features

### Metadata Remover
Strip hidden metadata from images before sharing them. Uploads an image, recreates it from raw pixel data (removing all EXIF, GPS, camera info, and other embedded metadata), and lets you save the clean copy.

**Supported formats:** JPEG, PNG, WebP, BMP, TIFF

## Building

Requires [Rust](https://rustup.rs).

```bash
git clone https://github.com/your-username/PrivacyToolKit
cd PrivacyToolKit
cargo run --release
```

## Development

```bash
cargo build       # debug build
cargo run         # build and run
cargo test        # run tests
cargo clippy      # lint
cargo fmt         # format
```

## Roadmap

- [ ] File encryption
- [ ] PDF metadata remover
- [ ] Document metadata remover (Word, Excel, etc.)

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting a pull request.
