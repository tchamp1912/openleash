# OpenLeash - Architecture

Permission and access management for AI agents. Keep your AI agent secure—built for [OpenClaw](https://github.com/openclaw/openclaw) with IT-style access controls.

## Overview

OpenLeash provides a secure, auditable system for managing what AI agents can access:

- **Secrets**: API keys and credentials brokered via macOS Keychain.
- **Packages**: Scoped package installation (pip, npm, brew) with session-aware tasks.
- **Execution**: Agents execute commands directly in sandbox using PATH provided by daemon.

### Why This Exists

AI agents need access to tools and secrets to complete missions. Giving unrestricted access is a security risk. This project provides:

1. **Sandbox Gap**: Agents run in restricted contexts (e.g. `sandbox-exec`) and request capabilities from the trusted `openleashd` via a Unix Domain Socket.
2. **Rationale-based Requests**: Agents must explain *why* they need access for every request.
3. **Session-Aware Tasks**: Unified environments for specific missions, with automatic atomic cleanup.
4. **Human-in-the-Loop**: Seamless approval workflows via CLI and Telegram.
5. **Audit Ledger**: A hash-chained (SHA-256) immutable record of all system actions.

## Architecture

The system is implemented as a **Rust workspace** with a **gRPC API**. The daemon (`openleashd`) hosts the services; the CLI (`openleash`) and other clients use the `openleash-client` crate to call them.

```
┌─────────────────────────────────────────────────────────────┐
│                    OpenClaw / Agent                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           openleash-client (Rust SDK)                 │  │
│  │  • request_package()                                 │  │
│  │  • request_secret() / store_secret()                 │  │
│  │  • get_task_environment()                           │  │
│  │  • start_task() / end_task()                         │  │
│  └──────────────────┬───────────────────────────────────┘  │
└─────────────────────┼───────────────────────────────────────┘
                      │ gRPC (tonic) over UDS / TCP
                      ▼
┌─────────────────────────────────────────────────────────────┐
│              openleashd (daemon)                                │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  RequestService / TaskService / ApprovalService      │  │
│  │  • Policy Engine (Regex + Priority)                  │  │
│  │  • Approval Scopes (Once, Task, Permanent)           │  │
│  │  • Hash-chained Audit Ledger                        │  │
│  │  • Managed task lifecycles                           │  │
│  └──────┬────────────────────────────────┬──────────────┘  │
│         │                                │                  │
│    ┌────▼────────┐              ┌───────▼────────┐        │
│    │ openleash-db │              │ openleash-venv   │        │
│    │ (SQLite)    │              │ (Managed Scopes)│        │
│    └─────────────┘              └───────┬────────┘        │
│                                         │                  │
│                        ┌────────────────▼──────────────┐  │
│                        │ Backends (Pip, NPM, Brew,     │  │
│                        │ Keychain, Telegram)           │  │
│                        └───────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Crate Layout

| Crate | Role |
|-------|------|
| **openleash-core** | Shared domain models, policy engine, and configuration schema. |
| **openleash-api** | gRPC API definition (.proto) and generated bindings. |
| **openleash-db** | SQLite persistence layer with hash-chain integrity. |
| **openleash-venv** | Unified lifecycle management for isolated task scopes. |
| **openleash-backend-*** | Pluggable implementations for Pip, NPM, Brew, Keychain, etc. |
| **openleash-client** | Async Rust SDK supporting UDS bridge. |
| **openleashd** | The trusted daemon; manages the state machine and brokered execution. |
| **openleash** | Management CLI for initialization, approvals, and manual requests. |

## gRPC API

### RequestService
- `RequestPackage`: Install a package into a task-scoped environment.
- `RequestSecret`: Fetch a secret from the Keychain backend.
- `StoreSecret`: Securely save a new credential.

### TaskService
- `StartTask`: Establish a scoped environment with a mandatory TTL.
- `EndTask`: Atomic teardown of an environment and its associated permissions.
- `GetTaskEnvironment`: Get the PATH/bin directory for a task to enable direct command execution.

### ApprovalService
- `ListPendingApprovals`: View requests waiting for human review.
- `Approve`: Grant access with an optional scope override.
- `Deny`: Explicitly block a request.

### AuditService
- `QueryAuditLogs`: Retrieve the history of all brokered actions.

## Security & Privilege Model

OpenLeash bridges the **Sandbox Gap**. Agents run in restricted contexts (e.g. macOS Seatbelt) and "request" capabilities from the non-sandboxed `openleashd`.

- **Approval Scopes**: Permissions can be granted `Once` (single request), for a `Task` (duration of the session), or `Permanent` (persisted in DB).
- **Direct Execution**: Agents execute commands directly in their sandbox using PATH provided by `GetTaskEnvironment`. The daemon only brokers resources (packages, secrets), not command execution.
- **Hash-Chaining**: Every audit event includes a hash of the previous event, ensuring the history cannot be tampered with without detection.
