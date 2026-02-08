# Leash AI

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

Leash AI is a **security-first privilege and access management layer** designed for AI agents. It acts as a controlled "bridge" between restricted agent sandboxes and the capable system host, providing scoped environments, secure tool installation, and audited credential access.

## 🚀 Key Features

- **Session-Aware Tasks**: Create time-scoped, isolated workspaces (venvs) that are automatically torn down after a mission.
- **Portable Backends**:
  - **Python**: Dedicated virtual environments.
  - **Node.js**: Global redirects into scoped prefixes.
  - **Homebrew**: Standalone, portable Homebrew instances bootstrapped inside the task scope.
- **Secure Key Management**: Inject secrets from the macOS Keychain directly into child process environments—secrets never touch the agent's disk or logs.
- **Fail-Closed Architecture**: Designed to run across a "Sandbox Gap" via Unix Domain Sockets.

## 🛠 Quick Start

### 1. Prerequisites
- **Rust**: Latest stable version.
- **macOS**: Required for the Keychain backend (v0).
- **Protobuf**: `brew install protobuf`

### 2. Run the Daemon
The daemon (`leashd`) manages the policies, database, and installations.
```bash
# Build the project
cargo build

# Start the daemon
./target/debug/leashd
```

### 3. Use the CLI
In another terminal (or from your agent):
```bash
# Start a task (creates a scoped environment)
leash task start --name "Data Analysis" --base-path /tmp/agent-work --ttl 3600

# Install a package into that task's scope
leash request install --manager pip --package pandas --task-id <TASK_ID>

# Run a script with a secret injected
leash exec --task-id <TASK_ID> --secret API_KEY=openai/api-key -- python my_agent.py
```

## 📖 Documentation

- [**Architecture**](docs/ARCHITECTURE.md): Deep dive into the multi-crate structure and gRPC design.
- [**Security Model**](docs/SECURITY_DESIGN.md): Detailed explanation of the "Sandbox Gap" and elevation flow.
- [**Extensibility Guide**](docs/EXTENSIBILITY.md): How to add new package managers or secret backends.

## ⚠️ Current Status: Alpha

Leash AI is under active development. 
- **Implemented**: Task lifecycles, Scoped Pip/NPM/Brew backends, Keychain backend, UDS IPC, `leash exec` injection.
- **In Progress**: Policy engine evaluation, immutable hash-chained audit logs, human-in-the-loop approval workflows.

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## 📄 License

Leash AI is released under the [Apache License 2.0](LICENSE).