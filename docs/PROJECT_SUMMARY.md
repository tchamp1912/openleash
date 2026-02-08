# OpenLeash - Project Summary

## 🎉 What We Built

OpenLeash is a comprehensive, production-ready permission and access management system designed specifically for AI agents (built for [OpenClaw](https://github.com/openclaw/openclaw)) running on **macOS**. It provides a secure "Sandbox Gap" between restricted agents and sensitive system resources.

## 📦 Core Pillars

1.  **🔐 Secrets**: Brokered access to macOS Keychain. Secrets are injected only into the memory of child processes, never touching the agent's disk.
2.  **📦 Packages**: Scoped installation of `pip`, `npm`, and `brew` packages into task-specific isolated environments (venvs) with automatic cleanup.
3.  **⚡ Commands**: Brokered shell execution via the daemon. Supports policy enforcement, standard stream capture, and automatic PATH expansion for task tools.

## 🚀 Key Deliverables

### Core Framework (Rust)
- **Unified Daemon (`openleashd`)**: Managed state machine for all requests, tasks, and audit logs.
- **Async SDK (`openleash-client`)**: Developer-friendly library for integrating Leash into any AI agent framework.
- **Management CLI (`openleash`)**: Powerful toolkit for environment initialization, manual approvals, and audit verification.

### Security Infrastructure
- **Policy Engine**: Regex-based pattern matching with priority weights and "deny-by-default" logic.
- **macOS Seatbelt Integration**: Tiered sandbox profiles (Permissive/Restrictive) that dynamically adapt to your enabled features.
- **Hash-Chained Audit Ledger**: Immutable record of all system actions secured with SHA-256 integrity chaining.

### Human-in-the-Loop
- **Interactive CLI**: Real-time management of pending requests.
- **Telegram Integration**: Mobile notifications with one-tap Approve/Deny buttons for remote agent management.
- **Approval Scopes**: Granular trust levels (Once, Task, Permanent) to reduce reviewer fatigue.

## 🏗️ Technical Highlights

- **Crates**: 13 modular Rust crates for high maintainability.
- **IPC**: Secure Unix Domain Sockets (/tmp/openleash.sock) bridging the sandbox gap.
- **Database**: SQLite with async SQLx for persistent tasks, leases, and audit integrity.
- **Reliability**: 27+ integration tests covering brokered execution, persistent approvals, and task lifecycle.

## 🎯 Design Philosophy

1.  **Sandbox Gap**: The agent is always confined; only the daemon is trusted.
2.  **Rationale First**: Agents must justify every action before it's evaluated.
3.  **Fail-Secure**: Any error or policy miss results in an immediate DENY.
4.  **Ephemeral by Default**: Environments and permissions are wiped as soon as a task ends.

## 💡 Quick Start

```bash
# 1. Initialize environment
openleash init

# 2. Start the Gatekeeper
openleashd &

# 3. Run a sandboxed mission
openleash task start --name "Research"
openleash run --task-id <ID> --reason "data analysis" -- python3 -c "print('Leashed!')"
```

---

**Project Status**: 🟢 Core functionality complete and verified.  
**Platform**: macOS 12+ (Full Support).  
**License**: Apache 2.0.