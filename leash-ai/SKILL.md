---
name: leash-ai
description: Managed security and resource brokering for AI agents on macOS. Use when the agent needs to establish a secure session, install scoped packages, or execute brokered commands with injected secrets across the Sandbox Gap.
---

# Leash AI Skill

Leash AI provides a secure "Capability Broker" for AI agents. This skill allows you to interact with system resources (Keychain, Pip, NPM, Brew, Shell) without exposing sensitive credentials to your context window.

## Core Workflow

### 1. Establish a Session (Task)
Always start a mission by creating a Leash Task. This provides a unique ID and an isolated directory for any tools you install.

```bash
leash task start --name "Mission Name" --ttl 3600
```
- Capture the `TASK_ID` from the output.

### 2. Install Tools on Demand
Request tool installation into your task scope. These tools are isolated from your host system.

```bash
leash request install --manager pip --package <name> --task_id <TASK_ID> --reason "<why>"
```

### 3. Secure Execution with Secret Injection
**CRITICAL**: Never read secrets into your own context window (e.g., using `leash request secret`). Instead, use `leash exec` to inject them directly into the environment of the tool you are running.

```bash
# Injects the secret 'anthropic/api-key' into the env var 'API_KEY' for your script
leash exec --task_id <TASK_ID> --secret API_KEY=anthropic/api-key -- python3 scrape.py
```
- **Security Benefit**: You (the LLM) never see the plain-text secret. It only exists in the memory of the script you execute.

### 4. Brokered Execution (No Secrets)
If no secrets are needed, use `leash run`. This automatically prepends your task binaries to the `PATH`.

```bash
leash run --task-id <TASK_ID> --reason "<rationale>" -- <command> <args>
```

### 5. Cleanup
Always terminate your task when the mission is finished to wipe the isolated environment and revoke all task-scoped permissions.

```bash
leash task end --task-id <TASK_ID>
```

## Security Best Practices
- **In-Memory Secrets Only**: Always use `leash exec` for credentials.
- **Provide clear rationales**: Every `reason` field is logged in a hash-chained audit ledger.
- **Fail-Secure**: If a command is denied, request human approval or adjust `~/.leash/policies.yaml`.

## Tool Reference
See [tools.json](references/tools.json) for a structured list of available commands and their parameters.
