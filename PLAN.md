# OpenLeash - Implementation Plan

This document outlines a high-level plan for implementing the OpenLeash project in Rust. The plan is broken into phases and parallel workstreams, designed to allow multiple agents to tackle different components simultaneously.

All implementation details MUST adhere to the specifications laid out in `CORE_SPEC.md` and the conceptual guides in the `docs/` directory.

## v0 Scope: Scoped Package Management

The initial implementation (v0) will focus exclusively on the **scoped package installation** use case. The primary goal is to deliver a robust and secure `openopenleash request install` flow.

*   **Priority 1:** `pip` installations into a dedicated `.venv`.
*   **Priority 2:** `npm` installations into a project-local `node_modules`.
*   **Out of Scope for v0:** Secret management, general command execution (`openopenleash exec`), Homebrew. These will be addressed in future milestones.

## Guiding Principle: Parallel Development

The project will be structured as a [Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) with multiple crates. This allows for clear separation of concerns and enables agents to work on independent crates with minimal friction. Each workstream corresponds to one or more crates within the workspace.

## Phase 1: Foundation (Contracts & Types)

This phase is foundational and must be completed first, but the two workstreams can be done in parallel. The goal is to establish the core, shared data structures and API contracts that all other components will depend on.

### Workstream 1: Core Types Crate (`openleash-core`)
*   **Agent Focus:** Data Structures Specialist
*   **Task:** Create a new crate, `openleash-core`, that defines all shared data structures and domain types.
*   **Outputs:**
    *   Rust structs for `Policy`, `ApprovalRequest`, `Lease` etc. based on `CORE_SPEC.md`.
    *   The primary `CapabilityGrant` (JWT claims) struct.
    *   The `AuditEvent` struct.
    *   Shared error types and enums (e.g., `ResourceType`, `RequestState`, `PackageManager`).
*   **Dependencies:** None (This is the root dependency for other crates).

### Workstream 2: API Contract Crate (`openleash-api`)
*   **Agent Focus:** API Design Specialist
*   **Task:** Create a new crate, `openleash-api`, to define the gRPC API contract for the v0 use case.
*   **Outputs:**
    *   `.proto` files defining the `RequestService` (with a focus on the `RequestPackage` RPC), `ApprovalService`, and `AuditService`.
    *   Build script (`build.rs`) using `tonic-build`.
*   **Dependencies:** `openleash-core`.

---

## Phase 2: Core Implementation (Parallel Sprints)

Once Phase 1 is complete, these workstreams can be developed largely in parallel.

### Workstream 3: Daemon Crate (`openleashd`)
*   **Agent Focus:** Main Application & Logic Specialist
*   **Task:** Build the main daemon application that implements the gRPC services for package management.
*   **Outputs:**
    *   A binary crate `openleashd`.
    *   Implementation of the `RequestPackage` gRPC endpoint.
    *   The core state machine and policy evaluation logic for package requests.
    *   The **Lease Management** service for tracking and reclaiming expired package installs.
*   **Dependencies:** `openleash-core`, `openleash-api`, `openleash-db`, `openleash-backend` (trait).

### Workstream 4: Database Crate (`openleash-db`)
*   **Agent Focus:** Database & Persistence Specialist
*   **Task:** Implement the data persistence layer for requests, audit events, and leases.
*   **Outputs:**
    *   A library crate `openleash-db`.
    *   Functions to store and manage `Request`, `AuditEvent`, and `Lease` records in SQLite.
    *   Implementation of the append-only, hash-chaining logic for the audit log.
*   **Dependencies:** `openleash-core`, `sqlx`.

### Workstream 5: Package Backend Crates (`openleash-backend-*`)
*   **Agent Focus:** Backend & Integration Specialist(s)
*   **Task:** Implement the `PackageBackend` trait according to the Backend Contract. This is the highest priority for v0.
*   **Outputs:**
    *   A shared `openleash-backend` crate defining the `PackageBackend` trait.
    *   **(v0 Priority 1)** A `openleash-backend-pip` crate for installing packages into a `.venv`.
    *   **(v0 Priority 2)** A `openleash-backend-npm` crate for installing packages into `node_modules`.
*   **Dependencies:** `openleash-core`.

### Workstream 6: Client SDK Crate (`openleash-client`)
*   **Agent Focus:** Client Library Specialist
*   **Task:** Build the user-friendly, async Rust client for the package request flow.
*   **Outputs:**
    *   A library crate `openleash-client`.
    *   A `Client` struct with a well-defined `request_package(...)` method.
*   **Dependencies:** `openleash-api`, `openleash-core`.

### Workstream 7: Management CLI Crate (`openleash`)
*   **Agent Focus:** CLI Application Specialist
*   **Task:** Build the `openleash` command-line tool with a focus on package management.
*   **Outputs:**
    *   A binary crate `openleash`.
    *   The agent-facing `request install` subcommand.
    *   Admin-facing subcommands for `openleash policy` and `openopenleash audit`.
*   **Dependencies:** `openleash-client`, `clap`.

---

## Phase 3: Integration & Hardening

This phase begins once the parallel workstreams from Phase 2 are functionally complete for the `pip` use case.

*   **Task:** Integration testing, end-to-end (E2E) test suite development for the `request install` flow.
*   **Agent Focus:** Integration & QA Specialist
*   **Outputs:**
    *   A stable, tested application for `pip` installation management.
    *   Finalized configuration and documentation for v0.

---

## Future Phases (Post-v0)

The following components from the original plan are deferred to maintain a tight focus for v0. The architecture established in v0 should make adding them straightforward.

*   **Secret Management Backends**: `openleash-backend-keychain`, `openleash-backend-vault`, etc.
*   **CLI Command Execution**: The `RequestCommand` flow and `CliBackend` trait.
*   **Advanced Package Managers**: Homebrew support, which may require the Privileged Helper.
*   **Privileged Helper**: Implementation of the root-level helper for `sudo` commands or system-wide installs.
