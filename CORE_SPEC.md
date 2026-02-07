# Leash AI - Core Specification

This document is the canonical source of truth for the core architecture, state transitions, and data models of Leash AI. It is language-agnostic and serves as the primary guide for implementation, testing, and validation. All other design documents are subordinate to this specification.

## 1. Core State Machine

Every request for a resource (secret, package, or command) follows this canonical state machine.

### States

*   **RECEIVED**: The initial state when a request is first received by the daemon.
*   **VALIDATED**: The request has been successfully parsed, and its syntax and basic parameters are well-formed.
*   **AUTO_APPROVED**: The request matched a policy that allows for automatic approval without human intervention.
*   **PENDING_APPROVAL**: The request matched a policy that requires human approval. The system is now waiting for a response from an approver.
*   **DENIED**: The request was denied, either by policy, by an approver, or due to a system error. This is a terminal state for a request that is not granted.
*   **TOKEN_ISSUED**: For secrets, a signed capability grant (token) has been generated and sent to the client.
*   **EXECUTED**: For packages or commands, the operation has been successfully executed by a backend.
*   **COMPLETED**: The resource has been successfully accessed or used. For tokens, this means the token was consumed. This is a terminal state for a successful request.
*   **EXPIRED**: A token was issued but not used within its `expires_at` window. This is a terminal state.
*   **REVOKED**: An approved request or an issued token was explicitly revoked by a user before it was used or expired. This is a terminal state.

### State Transitions

```
           +------------+
           |  RECEIVED  |
           +-----+------+
                 |
                 v
           +------------+
           | VALIDATED  |
           +-----+------+
                 |
  +--------------+--------------+
  |              |              |
  v              v              v
+-------------+  +-------------+  +------------------+
| AUTO_APPROVED |  |   DENIED    |  | PENDING_APPROVAL |
+-------------+  +-------------+  +--------+---------+
  |                                        |
  |             +--------------------------+-------------------+
  |             |                          |                   |
  v             v                          v                   v
+------------------------+         +---------------+   +---------------+
| TOKEN_ISSUED (Secrets) |         | DENIED (User) |   | REVOKED (User)|
|    OR                  |         +---------------+   +---------------+
| EXECUTED (Pkg/Cmd)     |
+-------------+----------+
              |
              |
              v
        +-----------+
        | COMPLETED |
        +-----------+

```

### Invariants (Tied to Threat Model)

1.  **Fail-Secure by Default**: Any error during parsing, validation, policy evaluation, or backend execution MUST transition the state to `DENIED`. The system must never grant access if there is ambiguity.
2.  **Scoped Grants**: All grants (`TOKEN_ISSUED` or `EXECUTED`) MUST be scoped to a single resource, a single operation, and a single agent instance.
3.  **Token Atomicity**: A capability token cannot be replayed. It is invalidated upon first use (`COMPLETED`), expiration (`EXPIRED`), or revocation (`REVOKED`).

## 2. Policy Evaluation

Policy evaluation MUST be deterministic, auditable, and easy to reason about.

*   **Match Model**: **First-match**. Policies are evaluated in a deterministic order. The first policy in the list that matches the request context is chosen.
*   **Conflict Resolution**: **Deny wins**. If the first matching policy has a `permission_level` of `deny`, the request is immediately `DENIED`. This aligns with the deny-by-default security posture.
*   **Priority Rules**: Policies can have an optional integer `priority` field. Policies with a higher `priority` value are evaluated first. If two policies have the same priority, they are evaluated in the order they appear in the configuration.
*   **TTL (Time-To-Live) Rules**: The final TTL for any granted resource is the **minimum** of:
    1.  The TTL requested by the client.
    2.  The `max_ttl` specified in the matching policy.
    3.  The maximum TTL configured globally in the daemon.

A policy testing harness will be developed to allow administrators to "dry-run" a request against the policy set and see the evaluation outcome without granting access.

## 3. Authorization vs. Execution (Capability Grants)

The system enforces a strict separation between the **Authorization Layer** and the **Execution Layer**.

*   **Authorization Layer**: Evaluates policies and, if approved, produces a signed **Capability Grant** (a JWT). This grant is the sole output.
*   **Execution Layer**: A small, hardened service that does only two things:
    1.  Validates the integrity and claims of a presented Capability Grant.
    2.  Executes the specific backend operation described in the grant.

### Capability Grant (Token) Format

The token will be a JSON Web Token (JWT) with the following claims in its payload:

```json
{
  "iss": "leashd",                          // Issuer (the daemon)
  "sub": "instance:openclaw-prod-123",      // Subject (the agent instance)
  "aud": "leash-backend:secrets",           // Audience (the specific backend)
  "jti": "uuid-v4-string",                  // JWT ID (for one-time use)
  "iat": 1675790000,                        // Issued At
  "nbf": 1675790000,                        // Not Before
  "exp": 1675793600,                        // Expires At (iat + TTL)
  "resource_type": "secret",                // Type of resource
  "resource_id": "aws/prod/billing-api-key",// The specific resource identifier
  "operation": "read",                      // The permitted operation
  "policy_id": "policy-prod-aws-keys",      // The policy that approved this
  "approval_id": "approval-uuid-xyz"        // (Optional) ID of the manual approval
}
```

This token is signed by a key held only by the Authorization Layer. The Execution Layer verifies the signature using a public key.

## 4. Secure Approval Protocol

The manual approval system is a secure protocol, not just a UI workflow.

*   **Approval Request**: An immutable, signed data structure containing the full context of the access request:
    *   `approval_request_id` (unique)
    *   Hash of the original request data (agent, resource, rationale, etc.)
    *   Timestamp
    *   Link to relevant policy
*   **Approval Response**: A signed decision from an authorized approver, bound to the `approval_request_id` and the request hash. This prevents tampering or re-application to other requests.
*   **Replay Protection**: Both requests and responses contain nonces and timestamps, validated by the daemon to prevent replay attacks.
*   **Timeout Behavior**: If an approver does not respond within a configurable timeout (e.g., 15 minutes), the request transitions to `DENIED`. "Break-glass" procedures must be initiated through a separate, explicitly permissioned request.

## 5. Audit Log

The audit log is the immutable system of record for all events.

*   **Authoritative Store**: A local **SQLite database** will be the primary, queryable store for audit events. Events are written to an append-only table.
*   **Immutability Mechanism**: We will use **hash chaining**. An `integrity` table in SQLite will store a chain of hashes. Each entry will be a hash of the current audit event combined with the hash of the previous entry, creating a tamper-resistant ledger.
*   **Export Format**: For shipping and archival, logs can be exported as **JSON Lines** (`.jsonl`), with each line representing a single audit event.

## 6. Privilege Separation

The system will enforce strict privilege separation for sensitive operations.

*   **Daemon (`leashd`)**: Runs as a **non-privileged user** at all times. It handles authorization but never executes privileged operations itself.
*   **Privileged Helper**: A minimal, securely-installed helper process will be introduced. This helper runs as root (or with specific `sudo` permissions) and exposes a very small, non-networked API (e.g., a Unix socket) that accepts only pre-defined commands from `leashd` (e.g., `install_package(name, version)`, `run_audited_sudo(command_hash)`). This drastically reduces the attack surface of privileged code.

## 7. Backend Contract

All backends (plugins) for secrets, packages, or commands MUST adhere to a strict contract.

*   **Idempotency**: Operations like `retrieve` should be idempotent. `install` should be able to handle cases where the resource already exists.
*   **Input Canonicalization**: Backends MUST canonicalize resource paths/names to prevent traversal attacks (e.g., `../` in a secret path).
*   **No Sensitive Logging**: Backends MUST NEVER log the content of secrets or other sensitive payloads.
*   **Capability Declaration**: Each backend will declare its capabilities (e.g., `supports_ttl`, `supports_delete`). The policy engine will use this to validate policies.
*   **Error Taxonomy**: Backends must return errors from a pre-defined set (e.g., `ResourceNotFound`, `PermissionDenied`, `BackendMisconfigured`) so that the core system can react appropriately.

## 8. API Contract

The daemon exposes a core API for clients.

*   **Transport**: gRPC will be the primary transport for its performance and schema-driven nature.
*   **Endpoints**:
    *   `RequestService.RequestSecret(Request)`
    *   `RequestService.RequestPackage(Request)`
    *   `RequestService.RequestCommand(Request)`
    *   `ApprovalService.Approve(ApprovalResponse)`
    *   `AuditService.Query(AuditQuery)`
    *   `HealthService.Check(HealthRequest)`
*   **Schemas**: All requests and responses will be defined via Protocol Buffers. IDs (`request_id`, `approval_id`, `token_id`) will be stable UUIDs.
*   **Error Model**: gRPC status codes will be used to signal the outcome class (e.g., `PERMISSION_DENIED`, `INVALID_ARGUMENT`, `UNAVAILABLE`). Custom error details will be provided in the response payload.
*   **Idempotency**: `Request` messages will contain a unique `request_id`. Clients can safely retry requests with the same `id` without causing duplicate operations. The daemon will track `request_id`s and return the original result for retried requests.
