# Contributing to HyperFrames Rust

Thank you for your interest in contributing to **HyperFrames Rust** (`hyperframes`)!

## Code Standards
- **Rust Edition**: Rust 2024 edition (`rust-version = "1.85"`).
- **Linter**: Zero Clippy warnings (`cargo clippy -- -D warnings`).
- **Formatting**: Format code with `cargo fmt`.
- **Testing**: All unit and integration tests must pass (`cargo test`).

## Submitting Pull Requests
1. Fork the repository and create your feature branch: `git checkout -b feature/my-new-feature`.
2. Commit your changes with clear, descriptive commit messages.
3. Verify tests and lints pass: `cargo test && cargo clippy -- -D warnings`.
4. Push to your branch and submit a Pull Request.

## Reporting Issues
Please include:
- Operating system and architecture.
- Chrome/Chromium and FFmpeg versions.
- Sample HTML reproduction case when reporting rendering bugs.
