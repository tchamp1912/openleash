# Leash AI - Project Roadmap

## Phase 1: Foundations & Core (v0.1.0) - 🟢 COMPLETE

**Goal**: Deliver the 3-pillar security model (Secrets, Packages, Commands) for OpenClaw on macOS.

### ✅ Achievements
- [x] Modular Cargo Workspace (13 crates)
- [x] gRPC API Contract over UDS/TCP
- [x] **Pip, NPM, and Portable Brew** backends
- [x] **macOS Keychain** secret brokering
- [x] **Brokered CLI Execution** with output capture
- [x] **Policy Engine** (Regex + Priority + Default Deny)
- [x] **Approval Scopes** (Once, Task, Permanent)
- [x] **Hash-Chained Audit Ledger** (SHA-256)
- [x] **Telegram Integration** for remote approvals
- [x] **Tiered Sandbox Profiles** (macOS Seatbelt)
- [x] Comprehensive Test Suite (27+ tests)

---

## Phase 2: OpenClaw Integration & DX (v0.2.0) - 🟡 IN PROGRESS

**Goal**: Seamlessly integrate Leash AI into the OpenClaw ecosystem and polish the developer experience.

### Core Features
- [ ] **OpenClaw MCP Server**: A native Model Context Protocol server for Leash AI.
- [ ] **Python SDK**: An idiomatic Python wrapper for the gRPC client.
- [ ] **SDK Examples**: Reference implementations for popular agent loops.
- [ ] **Auto-Reloading Policies**: Daemon reloads YAML rules without restart.

### Platform Support
- [ ] **Linux (v0.2.5)**: Support for APT/DNF and GNOME Keyring.

---

## Phase 3: Enterprise & Compliance (v0.3.0) - ⚪ PENDING

**Goal**: Hardening for production environments and compliance audits.

### Enhancements
- [ ] **Cloud Secret Backends**: AWS Secrets Manager & HashiCorp Vault.
- [ ] **Audit Export**: Automated JSON/CSV generation for SOC2/HIPAA compliance.
- [ ] **Policy Dry-Run**: Simulate the impact of a rule change before applying.
- [ ] **Web Dashboard**: Read-only UI for viewing audit logs and system status.

---

## Phase 4: Intelligence & Automation (v0.4.0) - ⚪ PENDING

**Goal**: Use usage data to improve security and reduce friction.

### Advanced Features
- [ ] **Smart Policy Suggestions**: Analyze rationale history to suggest new rules.
- [ ] **Anomaly Detection**: Flag unusual command patterns or secret access spikes.
- [ ] **Slack Integration**: Interactive approval workflow for Slack teams.
- [ ] **Biometric Approval**: Touch ID / Face ID integration for the `leash` CLI.

---

## Feedback & Contribution
We welcome community input! Please open a GitHub Discussion to influence the priority of these milestones.

**Last Updated**: 2025-02-08  
**Project Lead**: TOMMY