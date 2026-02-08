# OpenLeash - Security Design

OpenLeash is built on the principle of **Fail-Secure Isolation**. This document details the security mechanisms that protect sensitive resources from compromised or erratic AI agents.

## The Sandbox Gap

The core security innovation of OpenLeash is the **Sandbox Gap**. 

1.  **Confinement**: The AI agent is executed within a restricted context (e.g., macOS Seatbelt profile generated via `openopenleash init`). This profile denies all network access, broad filesystem access, and process execution by default.
2.  **Brokerage**: The only "hole" in the agent's sandbox is a Unix Domain Socket (/tmp/openleash.sock). The agent cannot access the internet or secrets directly; it must *request* these capabilities from the `openleashd` daemon.
3.  **Trust Boundary**: `openleashd` runs outside the agent's sandbox. It brokers resources (packages, secrets, PATH) but does not execute commands. Agents execute commands directly in their sandbox, ensuring no privilege escalation.

## Rationale-Based Access

Every request to the daemon must include a **Rationale string**.
- The daemon does not evaluate requests based on the identity of the agent alone.
- The rationale is recorded in the audit ledger and presented to human reviewers for any manual approval step.
- This creates a semantic audit trail: we know not just *what* was accessed, but *why* the agent claimed it needed it.

## Hash-Chained Audit Ledger

To prevent tampering with the action history, OpenLeash implements an immutable audit ledger:
- **Integrity**: Every audit event contains a `integrity_hash` (SHA-256).
- **Chaining**: The hash of event `N` is calculated as `hash(event_data + hash_of_event_N-1)`.
- **Persistence**: The current tail of the chain is stored in the `integrity` table in SQLite.
- **Verification**: The `openopenleash audit verify` command re-calculates the chain from the first entry to detect any deletions or modifications.

## Approval Scopes

OpenLeash balances security with developer productivity through granular approval scopes:

| Scope | Lifetime | Cleanup |
| :--- | :--- | :--- |
| **Once** | Single Request | Token invalidated immediately. |
| **Task** | Task Session | Automatically revoked when `openopenleash task end` is called. |
| **Permanent** | Indefinite | Must be manually revoked via database or CLI. |

Permissions are stored in the `approved_resources` table and checked *before* policy evaluation, allowing trusted agents to proceed without constant interruption.

## Secret Injection (Exec Flow)

When an agent needs a secret (e.g., an `OPENAI_API_KEY`):
1.  The agent calls `RequestSecret`.
2.  The daemon fetches the secret from the macOS Keychain.
3.  The secret is returned over the secure UDS.
4.  If using `openopenleash exec`, the secret is placed only in the environment variables of the child process.
5.  The secret **never touches the disk** and is cleared from the agent's memory as soon as the process terminates.

## Fail-Secure Defaults

- **Deny by Default**: If no policy explicitly allows a pattern, the request is denied.
- **Error = Deny**: Any internal error (DB failure, gRPC timeout) defaults to a `DENIED` status for the request.
- **Sanitized Inputs**: All resource IDs and command arguments are matched against strict regex patterns before execution.
