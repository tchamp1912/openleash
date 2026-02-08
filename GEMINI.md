# Project: OpenLeash

## Project Overview

OpenLeash is a permission and access management system for AI agents (e.g. [OpenClaw](https://github.com/openclaw/openclaw)). It provides secure, auditable access to secrets (API keys, tokens), package installation (pip, npm, brew) in task-scoped environments, and a path to human-in-the-loop approval and full audit trails. Agents run in a restricted sandbox and request capabilities from a trusted daemon over a Unix Domain Socket or TCP—bridging the "Sandbox Gap" so agents can get only what they need, when they need it, with time limits and rationale.

## Key Technologies

*   **Language:** Rust
*   **CLI:** clap (derive)
*   **API:** gRPC via tonic; Protocol Buffers in `crates/openleash-api/proto/`
*   **Database:** SQLite via sqlx (async, runtime-tokio)
*   **Configuration:** YAML for policies (openleash-core)
*   **Platform:** macOS Keychain for secrets (openleash-backend-keychain); portable venv and Homebrew for packages (openleash-venv, openleash-backend-brew).

## Architecture

The system is a **Cargo workspace** with a **gRPC API**. The daemon (`openleashd`) runs outside the agent’s sandbox and hosts the services; the CLI (`openleash`) and other clients use the `openleash-client` crate to call them.

*   **RequestService:** RequestPackage (pip/npm/brew), RequestSecret, StoreSecret.
*   **TaskService:** StartTask (create scoped environment with TTL), EndTask (atomic teardown and lease cleanup).
*   **AuditService / ApprovalService:** Defined in proto; audit and approval workflows are in progress.
*   **Backend traits:** `PackageBackend` (install/uninstall into scope) and `SecretBackend` (get/store). Implementations: pip, npm, brew, keychain; Telegram backend for notifications/approval is present in the workspace.
*   **openleash-venv:** Manages isolated scopes (venv, optional portable brew, NPM prefix) per task.
*   **openleash-db:** SQLite persistence for tasks, leases, and audit events.
*   **leash:** CLI for task/request operations and `openopenleash exec` (run a command with secrets injected into the environment).

## Crate Layout

| Crate | Role |
|-------|------|
| openleash-core | Domain models, errors, policy types, config, sandbox helpers. |
| openleash-api | gRPC API definition and generated bindings (proto). |
| openleash-db | SQLite: tasks, leases, audit. |
| openleash-venv | Scope lifecycle: venv, portable brew, NPM prefix. |
| openleash-backend | Traits: PackageBackend, SecretBackend. |
| openleash-backend-pip | Pip install into task scope. |
| openleash-backend-npm | NPM install with scoped prefix. |
| openleash-backend-brew | Standalone Homebrew inside task scope. |
| openleash-backend-keychain | macOS Keychain get/store. |
| openleash-backend-telegram | Telegram integration (notifications/approval). |
| openleash-client | Async Rust SDK (UDS and TCP). |
| openleashd | Daemon: RequestService, TaskService, backends, DB. |
| leash | CLI: tasks, requests, exec. |

## Building and Running

### Prerequisites

*   Rust toolchain (e.g. `rustup`). Use the resolver and toolchain implied by the workspace `Cargo.toml`.

### Build

```bash
cargo build
```

### Run the daemon

```bash
cargo run -p openleashd
```

Default: gRPC on `127.0.0.1:50051` and Unix Domain Socket at `/tmp/openleash.sock` (configurable via environment or config as implemented).

### Run the CLI

```bash
cargo run -p leash -- [subcommand]
```

Examples: task start/end, request install/secret, `openopenleash exec` with secret injection.

### Storing secrets (macOS Keychain)

```bash
security add-generic-password -s openleash -a "anthropic/api-key" -w "$ANTHROPIC_KEY"
```

### Testing

```bash
cargo test
```

Tests live in `crates/*/src/tests/` or `crates/*/tests/` as appropriate (e.g. openleash-core, openleash-db, openleashd).

### Linting and formatting

*   **Format:** `cargo fmt`
*   **Lint:** `cargo clippy`

## Development Conventions

*   **Rust:** Follow standard Rust style and workspace `Cargo.toml` (edition, resolver).
*   **Formatting:** `cargo fmt`.
*   **Linting:** `cargo clippy` (no warnings in CI).
*   **Testing:** `cargo test`; use existing test modules as reference (e.g. policy_tests, audit_tests, task_tests).
*   **Policy:** Policies are defined in YAML and loaded/evaluated via openleash-core (policy engine integration in openleashd is on the roadmap).
*   **Extensibility:** New backends implement the traits in `openleash-backend` and are registered in the daemon.

See `docs/ARCHITECTURE.md` for the full architecture and `CONTRIBUTING.md` for contribution and build details.
