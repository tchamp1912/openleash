# OpenLeash

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

**OpenLeash** is a security-first privilege and access management system designed for AI agents. It acts as a controlled "bridge" between restricted agent sandboxes and the capable system host, providing scoped environments, secure tool installation, and audited credential access.

Built for [OpenClaw](https://github.com/openclaw/openclaw) and compatible with any AI agent framework running on macOS.

## Key Features

- **Session-Aware Tasks**: Create time-scoped, isolated workspaces (venvs) that are automatically torn down after a mission
- **Portable Package Backends**:
  - **Python**: Dedicated virtual environments
  - **Node.js**: Global redirects into scoped prefixes
  - **Homebrew**: Standalone, portable Homebrew instances bootstrapped inside the task scope
- **Secure Key Management**: Inject secrets from the macOS Keychain directly into child process environments—secrets never touch the agent's disk or logs
- **Policy Engine**: Regex-based pattern matching with priority weights and "deny-by-default" logic
- **Human-in-the-Loop**: Interactive CLI and Telegram integration for real-time approval workflows
- **Audit Ledger**: Hash-chained (SHA-256) immutable record of all system actions
- **Fail-Closed Architecture**: Designed to run across a "Sandbox Gap" via Unix Domain Sockets

## Quick Start

### Installation

**Homebrew (Recommended):**

```bash
brew tap tchamp1912/openleash
brew install openleash
```

**Build from Source:**

```bash
git clone https://github.com/tchamp1912/openleash.git
cd openleash
cargo build --release
```

**Prerequisites:** macOS 12+, Rust ([rustup](https://rustup.rs/)), Protocol Buffers (`brew install protobuf`)

For detailed instructions, see [docs/INSTALLATION.md](docs/INSTALLATION.md).

### Running the Daemon

The daemon (`openleashd`) manages policies, database, and installations:

```bash
# Start the daemon (defaults to UDS at /tmp/openleash.sock)
cargo run -p openleashd

# Or build and run directly
./target/debug/openleashd
```

### Using the CLI

```bash
# Initialize your environment
cargo run -p openleash -- init

# Start a task (creates a scoped environment)
cargo run -p openleash -- task start --name "Data Analysis" --base-path /tmp/agent-work --ttl 3600

# Install a package into that task's scope
cargo run -p openleash -- request install --manager pip --package pandas --task-id <TASK_ID>

# Get task PATH and execute commands directly
eval $(cargo run -p openleash -- run --task-id <TASK_ID>)
python my_agent.py

# Or execute with secrets injected (secrets never touch disk)
cargo run -p openleash -- exec --task-id <TASK_ID> --secret API_KEY=openai/api-key -- python my_agent.py
```

### Storing Secrets (macOS Keychain)

```bash
security add-generic-password -s openleash -a "anthropic/api-key" -w "$ANTHROPIC_KEY"
```

## Documentation

- **[Installation Guide](docs/INSTALLATION.md)**: Comprehensive installation instructions, prerequisites, and troubleshooting
- **[Architecture](docs/ARCHITECTURE.md)**: Deep dive into the multi-crate structure and gRPC design
- **[Core Specification](docs/CORE_SPEC.md)**: Canonical source of truth for architecture, state transitions, and data models
- **[Security Design](docs/SECURITY_DESIGN.md)**: Detailed explanation of the "Sandbox Gap" and elevation flow
- **[Threat Model](docs/THREAT_MODEL.md)**: Security assumptions and threat analysis
- **[Extensibility Guide](docs/EXTENSIBILITY.md)**: How to add new package managers or secret backends
- **[OpenClaw Integration](docs/OPENCLAW_INTEGRATION.md)**: Integration guide for OpenClaw agents
- **[Features](docs/FEATURES.md)**: Implementation status tracker
- **[Roadmap](docs/ROADMAP.md)**: Future development plans

## Architecture Overview

OpenLeash is implemented as a **Rust workspace** with a **gRPC API**. The system consists of:

- **`openleashd`**: The trusted daemon that runs outside the agent's sandbox, handling policy evaluation, approvals, and resource brokering
- **`openleash`**: CLI tool for task management, approvals, and audit queries
- **`openleash-client`**: Async Rust SDK for integrating Leash into AI agent frameworks
- **Backend Crates**: Modular implementations for pip, npm, brew, and macOS Keychain
- **Core Crates**: Domain models, policy engine, database layer, and sandbox helpers

The agent runs in a restricted sandbox and communicates with the daemon via Unix Domain Socket (`/tmp/openleash.sock`) or TCP, bridging the "Sandbox Gap" securely.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed architecture documentation.

## Crate Structure

| Crate | Role |
|-------|------|
| `openleash-core` | Domain models, errors, policy types, config, sandbox helpers |
| `openleash-api` | gRPC API definition and generated bindings (proto) |
| `openleash-db` | SQLite: tasks, leases, audit |
| `openleash-venv` | Scope lifecycle: venv, portable brew, NPM prefix |
| `openleash-backend` | Traits: PackageBackend, SecretBackend |
| `openleash-backend-pip` | Pip install into task scope |
| `openleash-backend-npm` | NPM install with scoped prefix |
| `openleash-backend-brew` | Standalone Homebrew inside task scope |
| `openleash-backend-keychain` | macOS Keychain get/store |
| `openleash-backend-telegram` | Telegram integration (notifications/approval) |
| `openleash-client` | Async Rust SDK (UDS and TCP) |
| `openleashd` | Daemon: RequestService, TaskService, backends, DB |
| `openleash` | CLI: tasks, requests, exec |

## Current Status

**Alpha** - OpenLeash is under active development.

- **Implemented**: Task lifecycles, scoped Pip/NPM/Brew backends, Keychain backend, UDS IPC, secret injection, policy engine, audit ledger, Telegram integration
- **In Progress**: Full approval scope logic, audit export formats, Linux secret backend
- **Planned**: Web UI dashboard, Slack integration, additional backends

See [docs/FEATURES.md](docs/FEATURES.md) for detailed implementation status.

## Testing

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p openleash-core

# Run integration tests
cargo test --test integration_tests
```

## Development

### Formatting and Linting

```bash
# Format code
cargo fmt

# Lint code
cargo clippy
```

### Building

```bash
# Build all crates
cargo build

# Build release binaries
cargo build --release

# Build specific crate
cargo build -p openleashd
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Security

If you discover a security vulnerability, please do **not** use the public issue tracker. See [SECURITY.md](SECURITY.md) for instructions on how to report vulnerabilities privately.

## License

OpenLeash is released under the [Apache License 2.0](LICENSE).

## Acknowledgments

Built for the [OpenClaw](https://github.com/openclaw/openclaw) project and the broader AI agent community.
