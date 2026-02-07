# Leash AI - Architecture

Permission and access management for AI agents. Keep your AI agent on a leash—built for [OpenClaw](https://github.com/openclaw/openclaw) with IT-style access controls.

## 🎯 Overview

Leash AI provides a secure, auditable system for managing what AI agents can access:

- **🔐 Secrets**: API keys, tokens, passwords (OS Keychain, Vault, AWS Secrets Manager)
- **📦 Packages**: Temporary CLI tool installation (Homebrew, APT, Snap)
- **⚡ Commands**: Controlled CLI execution with policy enforcement

### Why This Exists

AI agents like OpenClaw need access to tools, secrets, and commands to complete tasks. But giving unrestricted access is a security nightmare. This project provides:

1. **Rationale-based Requests**: Agents must explain *why* they need access
2. **Human-in-the-Loop**: Approval workflows for sensitive operations
3. **Time-limited Access**: Auto-expiring permissions
4. **Audit Logging**: Complete trail of who accessed what and why
5. **Policy Engine**: Fine-grained control over what's allowed

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    OpenClaw Instance                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           Leash AI Client SDK                        │  │
│  │  • request_secret()                                  │  │
│  │  • request_package()                                 │  │
│  │  • execute_command()                                 │  │
│  └──────────────────┬───────────────────────────────────┘  │
└─────────────────────┼───────────────────────────────────────┘
                      │ HTTP/gRPC
                      ▼
┌─────────────────────────────────────────────────────────────┐
│              Leash daemon (leashd)                          │
│  ┌──────────────────────────────────────────────────────┐  │
│  │               Request Handler                        │  │
│  │  • Validate rationale                                │  │
│  │  • Evaluate policies                                 │  │
│  │  • Route to approval workflow                        │  │
│  │  • Issue time-limited tokens                         │  │
│  └──────┬────────────────────────────────┬──────────────┘  │
│         │                                │                  │
│    ┌────▼────────┐              ┌───────▼────────┐        │
│    │   Policy    │              │    Approval    │        │
│    │   Engine    │              │    Workflow    │        │
│    └────┬────────┘              └───────┬────────┘        │
│         │                                │                  │
│  ┌──────▼────────────────────────────────▼──────────────┐  │
│  │              Audit Logger                            │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────┬───────────────────────────────────────┘
                      │
        ┌─────────────┼─────────────────────┐
        │             │                     │
        ▼             ▼                     ▼
┌──────────────┐ ┌────────────┐   ┌────────────────┐
│   Secrets    │ │  Packages  │   │   CLI Exec     │
│   Backend    │ │  Backend   │   │   Backend      │
└──────────────┘ └────────────┘   └────────────────┘
        │             │                     │
   ┌────┴────┐   ┌────┴────┐          ┌────┴────┐
   ▼    ▼    ▼   ▼    ▼    ▼          ▼    ▼    ▼
 macOS Linux AWS brew apt snap      bash sudo PATH
Keychain      SM                   wrapper
```

## 🚀 Quick Start

### Installation

```bash
# Install Leash AI
pip install leash-ai

# Start the daemon
leash start

# Configure policies
leash policy add examples/policies/example-policies.yaml
```

### Basic Usage

```python
from leash_ai import LeashClient

# Initialize client
client = LeashClient(instance_id="my-openclaw-instance")

# Request secret access
secret = await client.request_secret(
    key="aws/dev/api-key",
    rationale="Need AWS credentials to list S3 buckets for backup verification"
)

# Request package installation
package = await client.request_package(
    name="jq",
    rationale="Need jq to parse JSON responses from API",
    temporary=True,
    ttl=3600  # Auto-remove after 1 hour
)

# Execute CLI command
result = await client.execute_command(
    command="git",
    args=["status"],
    rationale="Check repository status before deployment"
)
```

## 📋 Permission Workflow

### 1. Agent Makes Request

```python
secret = await client.request_secret(
    key="github/token/deploy",
    rationale="Need GitHub token to push release tags"
)
```

### 2. Daemon Evaluates Policies

```yaml
# Policy: Auto-approve GitHub tokens during business hours
- id: "github-tokens-auto"
  resource_type: "secret"
  permission_level: "allow_auto"
  secret_patterns: ["github/token/.*"]
  time_windows:
    - start_hour: 9
      end_hour: 17
      days_of_week: [0,1,2,3,4]
```

### 3. Decision Made

**Auto-Approved**: Agent gets immediate access
```
✓ Request auto-approved (matched policy: github-tokens-auto)
  Token expires in: 2 hours
```

**Requires Approval**: Human reviews request
```
⏳ Awaiting approval from: devops-team@company.com
   Rationale: Need GitHub token to push release tags
   Policy: github-tokens-approval-required
```

**Denied**: Policy blocks access
```
✗ Access denied
  Reason: Production secrets require admin approval
  Policy: production-secrets-restricted
```

## 🔒 Security Features

### Rationale Validation

Every request requires explanation:
```python
# ✓ Good rationale
rationale="Need AWS credentials to deploy updated Lambda functions for the user authentication service"

# ✗ Weak rationale (may be rejected)
rationale="testing"
```

Policies can enforce minimum rationale length and pattern matching.

### Time-Limited Access

All permissions expire:
```python
# Secret expires after 1 hour
secret = await client.request_secret(key="...", rationale="...", ttl=3600)

# Package auto-removed after use
package = await client.request_package(name="awscli", temporary=True, ttl=1800)
```

### Audit Trail

Every action is logged:
```
2025-02-07 14:23:15 | openclaw-123 | REQUEST  | secret:aws/dev/key | "Deploy Lambda functions"
2025-02-07 14:23:16 | openclaw-123 | APPROVED | secret:aws/dev/key | auto (policy: aws-dev-auto)
2025-02-07 14:23:17 | openclaw-123 | ACCESS   | secret:aws/dev/key | token:abc123
2025-02-07 15:23:17 | system       | EXPIRE   | secret:aws/dev/key | token:abc123
```

## 📝 Policy Examples

### Secret Access Policies

```yaml
# Production secrets require approval
- id: "prod-secrets-approval"
  resource_type: "secret"
  permission_level: "allow_with_approval"
  secret_patterns: ["production/.*", "prod/.*"]
  min_rationale_length: 50
  approvers: ["admin@company.com"]

# Dev secrets auto-approved
- id: "dev-secrets-auto"
  resource_type: "secret"
  permission_level: "allow_auto"
  secret_patterns: ["development/.*", "dev/.*"]
  max_ttl_seconds: 3600
```

### Package Installation Policies

```yaml
# Common dev tools auto-approved
- id: "dev-tools-auto"
  resource_type: "package"
  permission_level: "allow_auto"
  package_patterns: ["^(git|vim|curl|wget|jq)$"]
  allow_temporary_only: true

# Cloud CLIs require approval
- id: "cloud-cli-approval"
  resource_type: "package"
  permission_level: "allow_with_approval"
  package_patterns: ["awscli", "google-cloud-sdk", "azure-cli"]
```

### CLI Command Policies

```yaml
# Read-only commands auto-approved
- id: "readonly-auto"
  resource_type: "cli_command"
  permission_level: "allow_auto"
  command_patterns: ["^(ls|cat|grep|find)$"]
  denied_args: ["--delete", "-rf"]

# Destructive commands denied
- id: "deny-dangerous"
  resource_type: "cli_command"
  permission_level: "deny"
  priority: 100
  command_patterns: ["^(rm|dd|mkfs)$"]
```

## 🔌 Backend Support

### Secret Backends

| Backend | OS | Status | Features |
|---------|----|---------| ---------|
| macOS Keychain | macOS | ✅ Ready | Native encryption, Touch ID |
| GNOME Keyring | Linux | 🚧 Planned | KWallet, Secret Service |
| HashiCorp Vault | All | 🚧 Planned | Enterprise secrets |
| AWS Secrets Manager | All | 🚧 Planned | Cloud-native |
| File (encrypted) | All | ✅ Ready | Dev/testing only |

### Package Managers

| Manager | OS | Status |
|---------|----| --------|
| Homebrew | macOS/Linux | ✅ Ready |
| APT | Debian/Ubuntu | 🚧 Planned |
| DNF/YUM | RedHat/Fedora | 🚧 Planned |
| Snap | Linux | 🚧 Planned |

## 📊 Monitoring & Audit

### View Access Logs

```bash
# All requests today
leash audit list --today

# Specific instance
leash audit list --instance openclaw-123

# Denied requests only
leash audit list --status denied

# Export to JSON
leash audit export audit-2025-02.json
```

### Metrics

```bash
# Request statistics
leash stats
```

Output:
```
Total Requests:        1,247
Auto-Approved:           892  (71.5%)
Manual Approved:         234  (18.8%)
Denied:                  121  (9.7%)

Top Requesters:
  openclaw-deploy:       456
  openclaw-test:         234
  
Most Requested Secrets:
  aws/dev/api-key:       89
  github/token/ci:       67
```

## 🛠️ Development

### Project Structure

```
leash-ai/
├── src/leash_ai/
│   ├── backends/
│   │   ├── secrets/        # Secret storage backends
│   │   │   ├── base.py
│   │   │   ├── macos_keychain.py
│   │   │   └── vault.py
│   │   ├── package/        # Package manager backends
│   │   │   ├── base.py
│   │   │   ├── homebrew.py
│   │   │   └── apt.py
│   │   └── cli/            # CLI execution backends
│   │       ├── base.py
│   │       └── unix.py
│   ├── policies/           # Policy engine
│   │   ├── models.py
│   │   └── engine.py
│   ├── daemon/             # Permission daemon
│   │   ├── server.py
│   │   ├── approval.py
│   │   └── audit.py
│   └── client/             # Client SDK
│       └── sdk.py
├── examples/
│   ├── policies/
│   │   └── example-policies.yaml
│   └── usage_example.py
└── tests/
```

### Running Tests

```bash
# Install dev dependencies
pip install -e ".[dev]"

# Run tests
pytest

# With coverage
pytest --cov=leash_ai
```

## 🤝 Contributing

We welcome contributions! Areas needing help:

- [ ] Linux Secret Service backend
- [ ] APT package manager backend
- [ ] Windows support
- [ ] Web UI for approval workflow
- [ ] Slack/email notifications
- [ ] Policy testing framework
- [ ] Integration with OpenClaw core

## 📄 License

Apache 2.0 - See LICENSE file

## 🙏 Acknowledgments

Inspired by:
- AWS IAM policies
- Kubernetes RBAC
- HashiCorp Vault
- sudo/doas access control

---

**Built with ❤️ for the OpenClaw community — Leash AI**
