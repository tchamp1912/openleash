# Leash AI - Architecture

Permission and access management for AI agents. Keep your AI agent on a leash—built for [OpenClaw](https://github.com/openclaw/openclaw) with IT-style access controls.

## 🎯 Overview

Leash AI provides a secure, auditable system for managing what AI agents can access:

- **🔐 Secrets**: API keys and credentials brokered via macOS Keychain.
- **📦 Packages**: Scoped package installation (pip, npm, brew) with session-aware tasks.
- **⚡ Commands**: Brokered CLI execution with policy enforcement and output streaming.

### Why This Exists

AI agents need access to tools and secrets to complete missions. Giving unrestricted access is a security risk. This project provides:

1. **Sandbox Gap**: Agents run in restricted contexts (e.g. `sandbox-exec`) and request capabilities from the trusted `leashd` via a Unix Domain Socket.
2. **Rationale-based Requests**: Agents must explain *why* they need access for every request.
3. **Session-Aware Tasks**: Unified environments for specific missions, with automatic atomic cleanup.
4. **Human-in-the-Loop**: Seamless approval workflows via CLI and Telegram.
5. **Audit Ledger**: A hash-chained (SHA-256) immutable record of all system actions.

## 🏗️ Architecture

The system is implemented as a **Rust workspace** with a **gRPC API**. The daemon (`leashd`) hosts the services; the CLI (`leash`) and other clients use the `leash-ai-client` crate to call them.

```
┌─────────────────────────────────────────────────────────────┐
│                    OpenClaw / Agent                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           leash-ai-client (Rust SDK)                 │  │
│  │  • request_package()                                 │  │
│  │  • request_secret() / store_secret()                 │  │
│  │  • execute_command()                                 │  │
│  │  • start_task() / end_task()                         │  │
│  └──────────────────┬───────────────────────────────────┘  │
└─────────────────────┼───────────────────────────────────────┘
                      │ gRPC (tonic) over UDS / TCP
                      ▼
┌─────────────────────────────────────────────────────────────┐
│              leashd (daemon)                                │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  RequestService / TaskService / ApprovalService      │  │
│  │  • Policy Engine (Regex + Priority)                  │  │
│  │  • Approval Scopes (Once, Task, Permanent)           │  │
│  │  • Hash-chained Audit Ledger                        │  │
│  │  • Managed task lifecycles                           │  │
│  └──────┬────────────────────────────────┬──────────────┘  │
│         │                                │                  │
│    ┌────▼────────┐              ┌───────▼────────┐        │
│    │ leash-ai-db │              │ leash-ai-venv   │        │
│    │ (SQLite)    │              │ (Managed Scopes)│        │
│    └─────────────┘              └───────┬────────┘        │
│                                         │                  │
│                        ┌────────────────▼──────────────┐  │
│                        │ Backends (Pip, NPM, Brew,     │  │
│                        │ Keychain, Command, Telegram)  │  │
│                        └───────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## 📦 Crate Layout

| Crate | Role |
|-------|------|
| **leash-ai-core** | Shared domain models, policy engine, and configuration schema. |
| **leash-ai-api** | gRPC API definition (.proto) and generated bindings. |
| **leash-ai-db** | SQLite persistence layer with hash-chain integrity. |
| **leash-ai-venv** | Unified lifecycle management for isolated task scopes. |
| **leash-ai-backend-*** | Pluggable implementations for Pip, NPM, Brew, Keychain, etc. |
| **leash-ai-client** | Async Rust SDK supporting UDS bridge. |
| **leashd** | The trusted daemon; manages the state machine and brokered execution. |
| **leash** | Management CLI for initialization, approvals, and manual requests. |

## 🔌 gRPC API

### RequestService
- `RequestPackage`: Install a package into a task-scoped environment.
- `RequestSecret`: Fetch a secret from the Keychain backend.
- `StoreSecret`: Securely save a new credential.
- `ExecuteCommand`: Run a shell command via the daemon with PATH expansion.

### TaskService
- `StartTask`: Establish a scoped environment with a mandatory TTL.
- `EndTask`: Atomic teardown of an environment and its associated permissions.

### ApprovalService
- `ListPendingApprovals`: View requests waiting for human review.
- `Approve`: Grant access with an optional scope override.
- `Deny`: Explicitly block a request.

### AuditService
- `QueryAuditLogs`: Retrieve the history of all brokered actions.

## 🔒 Security & Privilege Model

Leash AI bridges the **Sandbox Gap**. Agents run in restricted contexts (e.g. macOS Seatbelt) and "request" capabilities from the non-sandboxed `leashd`.

- **Approval Scopes**: Permissions can be granted `Once` (single request), for a `Task` (duration of the session), or `Permanent` (persisted in DB).
- **PATH Expansion**: Brokered commands automatically include task binaries in their environment, avoiding the need for agents to know absolute paths.
- **Hash-Chaining**: Every audit event includes a hash of the previous event, ensuring the history cannot be tampered with without detection.
