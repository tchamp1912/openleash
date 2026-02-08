# Project: Leash AI

## Project Overview

Leash AI is a permission and access management system for AI agents (e.g. [OpenClaw](https://github.com/openclaw/openclaw)). It provides secure, auditable access to secrets (API keys, tokens), package installation (pip, npm, brew) in task-scoped environments, and a path to human-in-the-loop approval and full audit trails. Agents run in a restricted sandbox and request capabilities from a trusted daemon over a Unix Domain Socket or TCP—bridging the "Sandbox Gap" so agents can get only what they need, when they need it, with time limits and rationale.

## Key Technologies

*   **Language:** Rust
*   **CLI:** clap (derive)
*   **API:** gRPC via tonic; Protocol Buffers in `crates/leash-ai-api/proto/`
*   **Database:** SQLite via sqlx (async, runtime-tokio)
*   **Configuration:** YAML for policies (leash-ai-core)
*   **Platform:** macOS Keychain for secrets (leash-ai-backend-keychain); portable venv and Homebrew for packages (leash-ai-venv, leash-ai-backend-brew).

## Architecture

The system is a **Cargo workspace** with a **gRPC API**. The daemon (`leashd`) runs outside the agent’s sandbox and hosts the services; the CLI (`leash`) and other clients use the `leash-ai-client` crate to call them.

*   **RequestService:** RequestPackage (pip/npm/brew), RequestSecret, StoreSecret.
*   **TaskService:** StartTask (create scoped environment with TTL), EndTask (atomic teardown and lease cleanup).
*   **AuditService / ApprovalService:** Defined in proto; audit and approval workflows are in progress.
*   **Backend traits:** `PackageBackend` (install/uninstall into scope) and `SecretBackend` (get/store). Implementations: pip, npm, brew, keychain; Telegram backend for notifications/approval is present in the workspace.
*   **leash-ai-venv:** Manages isolated scopes (venv, optional portable brew, NPM prefix) per task.
*   **leash-ai-db:** SQLite persistence for tasks, leases, and audit events.
*   **leash:** CLI for task/request operations and `leash exec` (run a command with secrets injected into the environment).

## Crate Layout

| Crate | Role |
|-------|------|
| leash-ai-core | Domain models, errors, policy types, config, sandbox helpers. |
| leash-ai-api | gRPC API definition and generated bindings (proto). |
| leash-ai-db | SQLite: tasks, leases, audit. |
| leash-ai-venv | Scope lifecycle: venv, portable brew, NPM prefix. |
| leash-ai-backend | Traits: PackageBackend, SecretBackend. |
| leash-ai-backend-pip | Pip install into task scope. |
| leash-ai-backend-npm | NPM install with scoped prefix. |
| leash-ai-backend-brew | Standalone Homebrew inside task scope. |
| leash-ai-backend-keychain | macOS Keychain get/store. |
| leash-ai-backend-telegram | Telegram integration (notifications/approval). |
| leash-ai-client | Async Rust SDK (UDS and TCP). |
| leashd | Daemon: RequestService, TaskService, backends, DB. |
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
cargo run -p leashd
```

Default: gRPC on `127.0.0.1:50051` and Unix Domain Socket at `/tmp/leash.sock` (configurable via environment or config as implemented).

### Run the CLI

```bash
cargo run -p leash -- [subcommand]
```

Examples: task start/end, request install/secret, `leash exec` with secret injection.

### Storing secrets (macOS Keychain)

```bash
security add-generic-password -s leash-ai -a "anthropic/api-key" -w "$ANTHROPIC_KEY"
```

### Testing

```bash
cargo test
```

Tests live in `crates/*/src/tests/` or `crates/*/tests/` as appropriate (e.g. leash-ai-core, leash-ai-db, leashd).

### Linting and formatting

*   **Format:** `cargo fmt`
*   **Lint:** `cargo clippy`

## Development Conventions

*   **Rust:** Follow standard Rust style and workspace `Cargo.toml` (edition, resolver).
*   **Formatting:** `cargo fmt`.
*   **Linting:** `cargo clippy` (no warnings in CI).
*   **Testing:** `cargo test`; use existing test modules as reference (e.g. policy_tests, audit_tests, task_tests).
*   **Policy:** Policies are defined in YAML and loaded/evaluated via leash-ai-core (policy engine integration in leashd is on the roadmap).
*   **Extensibility:** New backends implement the traits in `leash-ai-backend` and are registered in the daemon.

See `docs/ARCHITECTURE.md` for the full architecture and `CONTRIBUTING.md` for contribution and build details.
