# Leash AI - Threat Model

**Version**: 1.0  
**Last Updated**: 2025-02-07  
**Status**: Security Review Required

## Executive Summary

This document identifies security threats to Leash AI and describes mitigations. Leash AI acts as a trusted intermediary between AI agents (OpenClaw instances) and sensitive resources (secrets, packages, system commands). This position makes it a high-value attack target.

**Critical Principle**: The permission system itself must not become a vulnerability amplifier. A compromised permission system is worse than no permission system.

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Trust Boundaries](#trust-boundaries)
3. [Assets & Threat Actors](#assets--threat-actors)
4. [Threat Catalog](#threat-catalog)
5. [Attack Trees](#attack-trees)
6. [Mitigations](#mitigations)
7. [Security Requirements](#security-requirements)
8. [Compliance Considerations](#compliance-considerations)

---

## System Overview

### Components

```
┌─────────────────────────────────────────────────────────────┐
│  External World                                             │
│  • Malicious users                                          │
│  • Compromised dependencies                                 │
│  • Network attackers                                        │
└─────────────────────────────────────────────────────────────┘
                           ▲
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│  OpenClaw    │  │  Human       │  │  Admin       │
│  Instance    │  │  Approver    │  │  User        │
│              │  │              │  │              │
│ Trust: LOW   │  │ Trust: MED   │  │ Trust: HIGH  │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                 │                 │
       └─────────────────┼─────────────────┘
                         │
      ═══════════════════════════════════════
        TRUST BOUNDARY: API/Authentication
      ═══════════════════════════════════════
                         │
                         ▼
              ┌──────────────────┐
              │  Leash           │
              │  Daemon          │
              │                  │
              │  Trust: HIGHEST  │
              └────────┬─────────┘
                       │
      ═══════════════════════════════════════
        TRUST BOUNDARY: Backend Access
      ═══════════════════════════════════════
                       │
       ┌───────────────┼───────────────┐
       ▼               ▼               ▼
┌────────────┐  ┌────────────┐  ┌────────────┐
│  Secret    │  │  Package   │  │  CLI       │
│  Backend   │  │  Backend   │  │  Backend   │
│            │  │            │  │            │
│ Secrets,   │  │ brew/apt   │  │ bash/sudo  │
│ Keys, etc  │  │            │  │            │
└────────────┘  └────────────┘  └────────────┘
```

### Data Flow

1. **Request**: OpenClaw → Daemon (policy evaluation, rationale)
2. **Approval**: Daemon → Human (if required)
3. **Token Issuance**: Daemon → OpenClaw (time-limited token)
4. **Resource Access**: OpenClaw → Backend (using token)
5. **Audit**: All steps logged

---

## Trust Boundaries

### Boundary 1: External → Daemon API

**Crossing**: HTTP/gRPC requests from OpenClaw instances

**Threats**:
- Unauthorized requests
- Request spoofing
- Man-in-the-middle attacks
- Replay attacks
- DDoS

**Controls**:
- Authentication (JWT, API keys)
- TLS encryption
- Rate limiting
- Request signing
- Nonce/timestamp validation

### Boundary 2: Daemon → Backends

**Crossing**: Internal calls to secret stores, package managers, shell

**Threats**:
- Privilege escalation
- Unauthorized backend access
- Secrets leakage
- Command injection

**Controls**:
- Least privilege execution
- Input validation
- Secrets in memory only (never logged)
- Sandboxed execution
- Backend authentication

### Boundary 3: Daemon ↔ Human Approver

**Crossing**: Approval workflow notifications/responses

**Threats**:
- Social engineering
- Approval bypass
- Notification spoofing
- Session hijacking

**Controls**:
- Multi-factor authentication
- Signed approval tokens
- Approval context display
- Session timeouts
- IP restrictions

---

## Assets & Threat Actors

### Assets (Ordered by Sensitivity)

1. **CRITICAL: Production Secrets**
   - Database passwords
   - API keys for production systems
   - Private keys, certificates
   - **Impact**: System compromise, data breach

2. **HIGH: Development Secrets**
   - Staging environment credentials
   - Test API keys
   - **Impact**: Development environment compromise

3. **HIGH: System Access**
   - Sudo/root execution capability
   - Package installation rights
   - **Impact**: Host compromise

4. **MEDIUM: Audit Logs**
   - Who accessed what and when
   - Rationale for access
   - **Impact**: Loss of accountability, compliance violation

5. **MEDIUM: Policy Configuration**
   - What's allowed/denied
   - Approval requirements
   - **Impact**: Unauthorized access patterns

6. **LOW: Usage Metrics**
   - Request counts
   - Performance data
   - **Impact**: Information disclosure

### Threat Actors

| Actor | Motivation | Capability | Priority |
|-------|------------|------------|----------|
| **Malicious User** | Steal secrets, escalate privileges | Low-Medium | HIGH |
| **Compromised OpenClaw Instance** | Lateral movement, data theft | Medium | HIGH |
| **Insider Threat** | Abuse legitimate access | High | MEDIUM |
| **External Attacker** | Remote exploit | Medium-High | HIGH |
| **Supply Chain Attack** | Backdoor via dependencies | Medium | MEDIUM |
| **Opportunistic Script Kiddie** | Automated scanning | Low | LOW |

---

## Threat Catalog

### STRIDE Analysis

#### Spoofing (Identity)

**T-001: OpenClaw Instance Impersonation**
- **Description**: Attacker pretends to be a legitimate OpenClaw instance
- **Impact**: Unauthorized secret/package access
- **Likelihood**: HIGH
- **Severity**: CRITICAL
- **Mitigations**:
  - REQUIRED: Instance authentication via JWT/API keys
  - REQUIRED: Instance registration with verification
  - RECOMMENDED: Certificate-based authentication
  - RECOMMENDED: IP whitelisting per instance

**T-002: Human Approver Impersonation**
- **Description**: Attacker spoofs approval notifications
- **Impact**: Unauthorized approvals
- **Likelihood**: MEDIUM
- **Severity**: HIGH
- **Mitigations**:
  - REQUIRED: MFA for approvers
  - REQUIRED: Signed approval responses
  - RECOMMENDED: Out-of-band approval confirmation
  - RECOMMENDED: Approval session binding

#### Tampering (Data)

**T-003: Request Modification**
- **Description**: Attacker modifies request in transit
- **Impact**: Access to unintended resources
- **Likelihood**: MEDIUM
- **Severity**: HIGH
- **Mitigations**:
  - REQUIRED: TLS for all communication
  - REQUIRED: Request signing
  - RECOMMENDED: Request integrity checks (HMAC)

**T-004: Policy Tampering**
- **Description**: Attacker modifies policy files
- **Impact**: Bypass security controls
- **Likelihood**: LOW (requires file access)
- **Severity**: CRITICAL
- **Mitigations**:
  - REQUIRED: File system permissions (600 for policies)
  - REQUIRED: Policy signature verification
  - REQUIRED: Audit log for policy changes
  - RECOMMENDED: Policy stored in versioned, immutable storage

**T-005: Audit Log Tampering**
- **Description**: Attacker deletes/modifies logs
- **Impact**: Loss of accountability
- **Likelihood**: MEDIUM
- **Severity**: HIGH
- **Mitigations**:
  - REQUIRED: Write-only audit logs
  - REQUIRED: Log integrity protection (signatures)
  - RECOMMENDED: Remote log aggregation (syslog, SIEM)
  - RECOMMENDED: Append-only storage

#### Repudiation

**T-006: Denial of Access**
- **Description**: OpenClaw instance denies making request
- **Impact**: Inability to prove malicious activity
- **Likelihood**: LOW
- **Severity**: MEDIUM
- **Mitigations**:
  - REQUIRED: Comprehensive audit logging
  - REQUIRED: Non-repudiable request signatures
  - RECOMMENDED: Cryptographic proof of request origin

#### Information Disclosure

**T-007: Secret Leakage in Logs**
- **Description**: Secrets written to logs
- **Impact**: Credential compromise
- **Likelihood**: HIGH (common mistake)
- **Severity**: CRITICAL
- **Mitigations**:
  - REQUIRED: Never log secret values
  - REQUIRED: Redaction of sensitive data in logs
  - REQUIRED: Log scrubbing (automated checks)
  - RECOMMENDED: Secrets held in memory only

**T-008: Secret Leakage in Error Messages**
- **Description**: Secrets in exception messages
- **Impact**: Information disclosure
- **Likelihood**: MEDIUM
- **Severity**: HIGH
- **Mitigations**:
  - REQUIRED: Generic error messages to clients
  - REQUIRED: Detailed errors only in secure logs
  - REQUIRED: Secret redaction in stack traces

**T-009: Timing Attacks**
- **Description**: Timing reveals if secret exists
- **Impact**: Information disclosure
- **Likelihood**: LOW
- **Severity**: LOW
- **Mitigations**:
  - RECOMMENDED: Constant-time comparisons
  - RECOMMENDED: Rate limiting

**T-010: Metadata Leakage**
- **Description**: Policy details revealed to unauthorized parties
- **Impact**: Security through obscurity loss
- **Likelihood**: MEDIUM
- **Severity**: LOW
- **Mitigations**:
  - RECOMMENDED: Minimal error details
  - RECOMMENDED: Policy inspection requires authorization

#### Denial of Service

**T-011: Request Flooding**
- **Description**: Overwhelm daemon with requests
- **Impact**: Service unavailability
- **Likelihood**: HIGH
- **Severity**: MEDIUM
- **Mitigations**:
  - REQUIRED: Rate limiting per instance
  - REQUIRED: Global rate limiting
  - RECOMMENDED: Request queue with priority
  - RECOMMENDED: Circuit breakers

**T-012: Resource Exhaustion via Package Installation**
- **Description**: Request installation of huge packages
- **Impact**: Disk/memory exhaustion
- **Likelihood**: MEDIUM
- **Severity**: MEDIUM
- **Mitigations**:
  - REQUIRED: Package size limits
  - REQUIRED: Installation timeout
  - REQUIRED: Disk space checks
  - RECOMMENDED: Installation quota per instance

**T-013: Approval Queue Flooding**
- **Description**: Spam approval requests
- **Impact**: Alert fatigue, approver overwhelm
- **Likelihood**: MEDIUM
- **Severity**: LOW
- **Mitigations**:
  - REQUIRED: Request limit per instance
  - RECOMMENDED: Duplicate request detection
  - RECOMMENDED: Approval batching

#### Elevation of Privilege

**T-014: Policy Bypass via Path Traversal**
- **Description**: Access secrets outside allowed paths
- **Impact**: Unauthorized secret access
- **Likelihood**: MEDIUM
- **Severity**: HIGH
- **Mitigations**:
  - REQUIRED: Path validation and canonicalization
  - REQUIRED: Reject `..` and absolute paths
  - REQUIRED: Chroot/jail for secret access

**T-015: Command Injection**
- **Description**: Inject shell commands via args
- **Impact**: Arbitrary code execution
- **Likelihood**: HIGH
- **Severity**: CRITICAL
- **Mitigations**:
  - REQUIRED: No shell execution (use exec directly)
  - REQUIRED: Argument whitelisting
  - REQUIRED: Escape/quote all arguments
  - RECOMMENDED: Run commands in sandbox

**T-016: Sudo Password Bypass**
- **Description**: Attempt to bypass sudo password requirements
- **Impact**: Unauthorized root access
- **Likelihood**: LOW
- **Severity**: CRITICAL
- **Mitigations**:
  - REQUIRED: Never store sudo passwords
  - REQUIRED: Sudo requires human approval
  - RECOMMENDED: Use sudo with NOPASSWD only for specific commands
  - RECOMMENDED: Audit all sudo usage

**T-017: Token Privilege Escalation**
- **Description**: Use token for unintended resource
- **Impact**: Unauthorized access
- **Likelihood**: MEDIUM
- **Severity**: HIGH
- **Mitigations**:
  - REQUIRED: Tokens bound to specific resource
  - REQUIRED: Token scope validation
  - REQUIRED: Token cannot be reused for different resource

---

## Attack Trees

### Attack: Steal Production Database Password

```
Goal: Obtain production database password
│
├─[OR] Compromise OpenClaw Instance
│  ├─[AND] Exploit OpenClaw vulnerability
│  │  └─ Mitigation: Keep OpenClaw updated
│  └─[AND] Request password with fake rationale
│     ├─ Bypass policy → MITIGATED: Policy requires approval for prod
│     └─ Social engineer approver → MITIGATED: Approval shows full context
│
├─[OR] Compromise Permission Daemon
│  ├─[AND] Exploit daemon vulnerability
│  │  └─ Mitigation: Security audits, dependency scanning
│  ├─[AND] Access daemon logs (password leaked?)
│  │  └─ Mitigation: Never log secrets
│  └─[AND] Access secret backend directly
│     └─ Mitigation: Backend requires authentication
│
├─[OR] Compromise Secret Backend
│  ├─[AND] Access keychain file
│  │  └─ Mitigation: OS-level encryption, file permissions
│  └─[AND] Exploit backend vulnerability
│     └─ Mitigation: Use hardened backends (Vault, AWS SM)
│
└─[OR] Intercept Token
   ├─[AND] MITM attack on daemon API
   │  └─ Mitigation: TLS required
   └─[AND] Token replay
      └─ Mitigation: Short token TTL, one-time use
```

### Attack: Execute Arbitrary Commands

```
Goal: Execute arbitrary commands as root
│
├─[OR] Command Injection
│  ├─[AND] Inject via command name
│  │  └─ Mitigation: Command whitelist
│  └─[AND] Inject via arguments
│     └─ Mitigation: No shell, argument validation
│
├─[OR] Request Sudo Access
│  ├─[AND] Fake emergency rationale
│  │  └─ Mitigation: Sudo requires approval + strong rationale
│  └─[AND] Bypass approval
│     └─ Mitigation: Approval workflow enforced
│
└─[OR] Install Malicious Package
   ├─[AND] Request installation of backdoor
   │  └─ Mitigation: Package name validation
   └─[AND] Replace legitimate package
      └─ Mitigation: Package manager signature verification
```

---

## Mitigations

### Defense in Depth

```
Layer 1: Prevention
├─ Authentication (JWT, API keys, certificates)
├─ Authorization (policy engine)
├─ Input Validation (all inputs sanitized)
└─ Encryption (TLS, at-rest)

Layer 2: Detection
├─ Audit Logging (all actions logged)
├─ Anomaly Detection (unusual patterns)
├─ Integrity Checks (file/policy/log verification)
└─ Monitoring (metrics, alerts)

Layer 3: Response
├─ Automatic Revocation (compromised instances)
├─ Incident Response (playbooks)
├─ Rollback (policy/config)
└─ Forensics (detailed audit trail)
```

### Security Controls Matrix

| Threat ID | Control Type | Implementation | Priority |
|-----------|--------------|----------------|----------|
| T-001 | Preventive | JWT authentication | CRITICAL |
| T-002 | Preventive | MFA for approvers | HIGH |
| T-003 | Preventive | TLS encryption | CRITICAL |
| T-004 | Preventive | Policy signing | HIGH |
| T-005 | Detective | Audit logging | CRITICAL |
| T-007 | Preventive | Secret redaction | CRITICAL |
| T-011 | Preventive | Rate limiting | HIGH |
| T-015 | Preventive | No shell execution | CRITICAL |
| T-017 | Preventive | Token binding | HIGH |

---

## Security Requirements

### MUST HAVE (Required for 1.0 Release)

1. **Authentication**
   - [ ] JWT-based instance authentication
   - [ ] API key support
   - [ ] Token expiration (max 24h)

2. **Authorization**
   - [ ] Policy engine with deny-by-default
   - [ ] Token scope binding (one token = one resource)
   - [ ] Time-limited permissions

3. **Encryption**
   - [ ] TLS 1.3 for all network communication
   - [ ] Secrets encrypted at rest (via backend)
   - [ ] Secure token generation (cryptographically random)

4. **Audit**
   - [ ] All requests logged (who, what, when, why)
   - [ ] All approvals logged
   - [ ] All secret access logged
   - [ ] Logs immutable (append-only)

5. **Input Validation**
   - [ ] Path canonicalization
   - [ ] Command argument validation
   - [ ] Rationale length/content validation
   - [ ] No shell execution (use exec)

6. **Secret Safety**
   - [ ] Never log secret values
   - [ ] Secrets in memory only
   - [ ] Automatic redaction in errors
   - [ ] Secure cleanup (zero memory)

### SHOULD HAVE (Recommended for Production)

1. **Advanced Authentication**
   - [ ] Certificate-based auth
   - [ ] MFA for human approvers
   - [ ] IP whitelisting per instance

2. **Enhanced Monitoring**
   - [ ] Anomaly detection
   - [ ] Alerting on suspicious patterns
   - [ ] Rate limit violations tracked

3. **Hardening**
   - [ ] Run daemon as non-root user
   - [ ] Principle of least privilege
   - [ ] Sandboxed command execution
   - [ ] Resource limits (CPU, memory, disk)

4. **Compliance**
   - [ ] Audit log retention policy
   - [ ] Compliance reporting (SOC2, HIPAA)
   - [ ] Data residency controls

### NICE TO HAVE (Future Enhancements)

1. **Zero Trust**
   - [ ] Continuous authentication
   - [ ] Device attestation
   - [ ] Behavioral analysis

2. **Advanced Features**
   - [ ] Secrets rotation automation
   - [ ] Break-glass emergency access
   - [ ] Approval delegation

---

## Compliance Considerations

### SOC 2 Type II

- **Access Control**: Policy engine enforces authorization
- **Audit Logging**: Complete audit trail of all access
- **Change Management**: Policy changes logged and versioned
- **Incident Response**: Automated revocation and alerting

### HIPAA

- **Access Logging**: All PHI access must be logged
- **Encryption**: PHI secrets encrypted at rest and in transit
- **Authentication**: Strong authentication for all access
- **Audit Trail**: Immutable logs for compliance reporting

### GDPR

- **Data Minimization**: Only essential data in logs
- **Right to Deletion**: Ability to purge instance data
- **Data Portability**: Export audit logs in standard format
- **Privacy by Design**: Security controls built-in

---

## Security Review Checklist

Before production deployment:

- [ ] **Threat model reviewed** by security team
- [ ] **Penetration testing** completed
- [ ] **Code review** with security focus
- [ ] **Dependency scanning** (no known CVEs)
- [ ] **Secrets scanning** (no hardcoded secrets)
- [ ] **TLS configuration** validated
- [ ] **Audit logging** tested
- [ ] **Incident response plan** documented
- [ ] **Security contacts** established
- [ ] **Disclosure policy** published

---

## Reporting Security Issues

**DO NOT** open public GitHub issues for security vulnerabilities.

Instead, email: security@openclaw.dev

Include:
- Description of the vulnerability
- Steps to reproduce
- Impact assessment
- Suggested fix (if available)

We will respond within 48 hours and aim to patch critical issues within 7 days.

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-02-07 | Initial threat model |

---

**Next Review**: Before 1.0 release or whenever architecture changes significantly
