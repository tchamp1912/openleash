# Leash AI - Threat Model

**Version**: 1.1  
**Last Updated**: 2025-02-08  
**Status**: Updated with Sandbox Gap & Hash-Chaining

## Executive Summary

This document identifying security threats to Leash AI and describes mitigations. Leash AI acts as a trusted intermediary between AI agents and sensitive resources.

**Critical Principle**: The **Sandbox Gap** ensures that even a fully compromised agent cannot access system resources without brokering through the daemon.

---

## Trust Boundaries

### 🌉 Boundary 1: The Sandbox Gap (Agent ↔ Daemon)
**Crossing**: gRPC requests over a Unix Domain Socket (`/tmp/leash.sock`).

**Threats**:
- Socket hijacking.
- Malicious rationale injection.
- Rationale spoofing.

**Controls**:
- **UDS Permissions**: Socket file is restricted to the agent's user.
- **Confinement**: Agent runs in a macOS Seatbelt profile that denies all syscalls except basic UDS IPC.
- **Mandatory Rationale**: Daemon rejects any request without a non-empty rationale.

### 🛡️ Boundary 2: Daemon ↔ System Backends
**Crossing**: Internal calls to Keychain, Pip, NPM, Brew, and Shell.

**Threats**:
- **Command Injection**: Attacker uses shell metacharacters in `execute_command` arguments.
- **Path Traversal**: Agent requests installation into a sensitive system directory.

**Controls**:
- **No-Shell Execution**: `leashd` uses `std::process::Command` without a shell wrapper.
- **Isolated Scopes**: Installations are strictly limited to `/tmp/leash-tasks/` via `VenvManager`.
- **Regex Enforcement**: Policy Engine validates all Resource IDs against strict whitelist patterns.

---

## Threat Catalog

### Tampering

**T-001: Audit Log Manipulation**
- **Description**: Attacker deletes log entries to hide malicious activity.
- **Impact**: Loss of accountability.
- **Mitigation**: **Hash-Chained Integrity**. Every log entry is chained to the previous one using SHA-256. `leash audit verify` detects any gaps or alterations in the history.

**T-002: Policy Bypass**
- **Description**: Attacker modifies the local YAML policy file.
- **Impact**: Permanent unauthorized access.
- **Mitigation**: **File Integrity**. Policies should be owned by `root` or a dedicated `leashd` user with `600` permissions.

### Information Disclosure

**T-003: Secret Leakage**
- **Description**: API keys are written to stdout or logs.
- **Mitigation**: **In-Memory Injection**. Secrets are never logged by the daemon and are injected directly into the environment of brokered processes.

---

## Security Requirements (v0.1.0 Status)

| Requirement | Status | Implementation |
| :--- | :---: | :--- |
| **Sandbox Isolation** | ✅ | Tiered macOS Seatbelt profiles. |
| **Immutable Audit** | ✅ | Hash-chained SHA-256 ledger in SQLite. |
| **Least Privilege** | ✅ | Task-scoped venvs and portable tool instances. |
| **Human Approval** | ✅ | Mandatory intervention for high-priority patterns. |
| **Input Validation** | ✅ | Regex-based policy matching. |