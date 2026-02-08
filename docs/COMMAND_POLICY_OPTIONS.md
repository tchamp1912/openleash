# Command Policy Enforcement Options

## Overview

After migrating to the Pure Resource Broker model, we removed command execution from the daemon. This document explores options for bringing back command policy enforcement while maintaining security.

## Option 1: Command Parser + Policy Check (Recommended)

### Approach
1. Parse command string using `shlex` crate
2. Extract command name (first token)
3. Check against policies using existing `PolicyEngine`
4. Execute if approved, deny otherwise

### Implementation Sketch

```rust
use shlex::split;
use leash_ai_core::models::{ResourceType, Decision};
use leash_ai_core::policy::PolicyEngine;

pub fn validate_and_execute_command(
    command_str: &str,
    policy_engine: &PolicyEngine,
    task_id: Option<&str>,
) -> Result<(), String> {
    // Parse command
    let parts = split(command_str)
        .ok_or("Failed to parse command")?;
    
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }
    
    let cmd_name = &parts[0];
    
    // Check policy
    match policy_engine.evaluate(ResourceType::Command, cmd_name) {
        Decision::Allow => {
            // Execute command
            execute_command(parts)
        }
        Decision::Deny(reason) => Err(reason),
        Decision::PendingApproval(scope) => {
            // Create approval request
            Err(format!("Command requires approval: {}", reason))
        }
    }
}
```

### Pros
- [PRO] Lightweight - no full interpreter needed
- [PRO] Works with existing policy engine
- [PRO] Can validate command name + arguments
- [PRO] Minimal dependencies (`shlex` is small)
- [PRO] Maintains security model (daemon validates, agent executes)

### Cons
- [CON] Doesn't handle complex shell features (pipes, redirects, etc.)
- [CON] Requires command to be passed as string (not ideal for complex scripts)

### Dependencies
```toml
[dependencies]
shlex = "1.3"  # MIT/Apache-2.0 licensed
```

## Option 2: Full Shell Parser (conch-parser)

### Approach
Use `conch-parser` for full POSIX shell parsing, then extract commands and check policies.

### Pros
- [PRO] Handles full POSIX shell syntax
- [PRO] Can parse complex commands with pipes, redirects, etc.
- [PRO] More accurate command extraction

### Cons
- [CON] More complex to integrate
- [CON] Larger dependency
- [CON] May be overkill for simple command validation

### Dependencies
```toml
[dependencies]
conch-parser = "0.1"  # MIT licensed
```

## Option 3: Embed Restricted Shell (shmy)

### Approach
Embed `shmy` Rust shell interpreter with hooks to intercept command execution.

### Pros
- [PRO] Full shell interpreter
- [PRO] Can intercept all command executions
- [PRO] Supports interactive mode

### Cons
- [CON] Different shell (not bash-compatible)
- [CON] Larger dependency
- [CON] More complex integration
- [CON] May not support all bash features

### Dependencies
```toml
[dependencies]
shmy = "0.1"  # Check license
```

## Option 4: Command Validation Layer

### Approach
Create a new `Command` resource type that validates commands before execution. The agent requests permission to execute a command, gets approval, then executes it directly.

### Flow
1. Agent calls `RequestCommand(command, args, reason, task_id)`
2. Daemon parses command, checks policy
3. If approved, returns approval token
4. Agent executes command directly (with token validation if needed)

### Pros
- [PRO] Maintains Pure Resource Broker model
- [PRO] Clear separation: daemon validates, agent executes
- [PRO] Can audit all command attempts
- [PRO] Works with existing approval system

### Cons
- [CON] Requires two-step process (request → execute)
- [CON] May be slower for simple commands

## Recommendation

**Option 1 (Command Parser + Policy Check)** is the best balance:

1. **Lightweight**: `shlex` is a small, well-maintained crate
2. **Fits architecture**: Works with existing policy engine
3. **Secure**: Daemon validates, agent executes (maintains Sandbox Gap)
4. **Simple**: Easy to integrate into existing codebase

### Implementation Plan

1. Add `shlex` dependency to `openleash-core`
2. Add `Command` back to `ResourceType` enum
3. Create `validate_command()` function that:
   - Parses command string
   - Extracts command name
   - Checks against policies
   - Returns `Decision`
4. Add `RequestCommand` API (optional - could also validate inline)
5. Update `openleash run` to validate commands before returning PATH

### Example Policy

```yaml
policies:
  - id: "allow-safe-commands"
    name: "Safe Commands"
    resource_type: Command
    priority: 10
    allowed_patterns:
      - "^ls$"
      - "^cat$"
      - "^grep$"
      - "^python3$"
      - "^node$"
    auto_approve: true
    default_scope: Once
```

## Security Considerations

- **Command Injection**: Use `shlex` to properly parse and escape commands
- **Argument Validation**: Could extend to validate arguments against patterns
- **Path Resolution**: Ensure commands are resolved from task PATH, not system PATH
- **Audit Trail**: Log all command validation attempts

## Next Steps

1. Evaluate `shlex` crate for command parsing
2. Design command validation API
3. Implement policy checking for commands
4. Add tests for command validation
5. Update documentation
