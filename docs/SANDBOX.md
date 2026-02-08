# Sandboxing in Leash AI (macOS Seatbelt)

Leash AI uses macOS **Seatbelt** (the native sandbox framework used by Safari and other system apps) to isolate AI agents. This provides a "Sandbox Gap" where the agent is strictly confined, and all sensitive operations must be brokered by the `leashd` daemon.

## Sandbox Profiles

Leash AI provides several tiered sandbox profiles out of the box. You can generate or customize these using the CLI.

### Tiered Profiles

| Profile | Level | Network | Description |
| :--- | :--- | :--- | :--- |
| `permissive-open` | Permissive | Allowed | Access to read all files; writes restricted to task scopes. |
| `permissive-closed` | Permissive | Blocked | Same as above, but no network access. |
| `restrictive-open` | Restrictive | Allowed | Access restricted to system libraries and the project directory. |
| `restrictive-closed` | Restrictive | Blocked | Maximum isolation; no network, minimal file access. |

## Feature-Aware Permissions

When you run `leash init`, Leash automatically detects your configuration and adds necessary "holes" to the sandbox for your enabled tools:

- **Pip/Python**: Allows execution of `/usr/bin/python3`.
- **Homebrew**: Allows access to `/opt/homebrew` binaries.
- **NPM/Node**: Allows execution of `node`.
- **macOS Keychain**: Allows IPC communication with `securityd`.

## Using the Sandbox

To run a command inside the sandbox, use the native macOS `sandbox-exec` tool:

```bash
sandbox-exec -f ~/.leash/agent.sb <your-command>
```

Or use the provided helper script:

```bash
./sandbox/run-sandboxed.sh <your-command>
```

## Management via CLI

You can list and generate profiles using the `leash sandbox` command:

```bash
# List available templates
leash sandbox list

# Generate a restrictive profile and save it
leash sandbox generate --profile restrictive-closed --output my_secure_agent.sb
```

## Anatomy of a Leash Profile

A Leash-generated profile typically includes:

1.  **Deny Default**: `(deny default)` - Everything is blocked unless explicitly allowed.
2.  **System Libs**: Allows reading `/usr/lib`, `/System/Library`, etc.
3.  **Leash Infrastructure**: Allows talking to `/tmp/leash.sock` (the daemon).
4.  **Task Scopes**: Allows full access to `/tmp/leash-tasks/` (where your virtual environments live).
5.  **Audit & Logs**: Allows writing to `stdout`/`stderr`.
