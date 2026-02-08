# Contributing to Leash AI

We love your input! We want to make contributing to Leash AI as easy and transparent as possible, whether it's:

- Reporting a bug
- Discussing the current state of the code
- Submitting a fix
- Proposing new features
- Becoming a maintainer

## Development Process

We use GitHub to host code, to track issues and feature requests, as well as accept pull requests.

### 🛠 Building from source

1.  Clone the repo.
2.  Install dependencies (Rust, Protobuf, macOS for Keychain).
3.  Build: `cargo build`.
4.  Run tests: `cargo test`.

### 🧪 Testing

Please ensure that your PR includes tests if you are adding new logic or backends. 
- Unit tests go in the same crate as the implementation.
- Integration tests can be run manually using the `sandbox/run-sandboxed.sh` helper.

## Pull Request Process

1.  Fork the repo and create your branch from `main`.
2.  If you've added code that should be tested, add tests.
3.  If you've changed APIs, update the documentation.
4.  Ensure the test suite passes.
5.  Make sure your code lints (`cargo fmt` and `cargo clippy`).

## 🛡 Security

If you find a security vulnerability, please do **not** open a public issue. See [SECURITY.md](SECURITY.md) for instructions on how to report vulnerabilities privately.

## 📜 License

By contributing, you agree that your contributions will be licensed under its Apache License 2.0.
