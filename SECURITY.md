# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability within Leash AI, please do **not** use the public issue tracker. Instead, please report it via one of the following methods:

- [Option 1: Email maintainers - add email here]
- [Option 2: GitHub Private Vulnerability Reporting]

Please include as much detail as possible, including steps to reproduce the issue.

## Security Model Assumptions

Leash AI is designed around the **Sandbox Gap** model. We assume:
1. The Agent is untrusted and restricted.
2. The Daemon (`leashd`) is trusted and has host privileges.
3. Communication occurs over a secure Unix Domain Socket.

## Known Limitations (v0 Alpha)

- **Plaintext UDS**: Communication over the Unix socket is not encrypted by default (relies on filesystem permissions for security).
- **macOS Focus**: The Keychain backend is currently macOS-only.
- **Auto-Approval**: Default policies are permissive. Users are encouraged to define strict YAML policies for production use.
