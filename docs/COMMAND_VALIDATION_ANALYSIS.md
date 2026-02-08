# Command Validation: Feasibility & Trade-off Analysis

## Current Model (Pure Resource Broker)

### How It Works Now
1. **`leash exec`**: Fetches secrets → executes command locally (no validation)
2. **`leash run`**: Returns PATH → agent executes directly (no validation)
3. **Agent**: Executes commands directly in sandbox
4. **Security**: Relies entirely on sandbox restrictions

### Current Security Posture
- [YES] **Sandbox Gap**: Agent can't access system resources without daemon
- [YES] **Package Isolation**: Packages installed in task scope
- [YES] **Secret Protection**: Secrets injected via `leash exec`, never touch disk
- [NO] **No Command Control**: Any command can be executed (within sandbox limits)
- [NO] **No Audit Trail**: Commands executed directly aren't logged/audited

## Proposed Model (Command Validation)

### How It Would Work
1. **Option A - Validation in `leash exec`**:
   ```rust
   // In leash exec
   let cmd_name = parse_command(&command[0])?;
   let decision = policy_engine.evaluate(ResourceType::Command, cmd_name)?;
   match decision {
       Decision::Allow => execute_command(),
       Decision::Deny => error!(),
       Decision::PendingApproval => create_approval_request(),
   }
   ```

2. **Option B - New `RequestCommand` API**:
   ```rust
   // Agent requests permission
   let approval = client.request_command("python3", args, reason, task_id)?;
   // Then executes locally
   execute_command_with_approval(approval)?;
   ```

## Feasibility Analysis

### [YES] Would It Work? **YES, but with limitations**

**Technical Feasibility:**
- [YES] `shlex` can parse simple commands reliably
- [YES] Policy engine already supports regex matching
- [YES] Can integrate into `leash exec` easily
- [YES] Can add `RequestCommand` API

**Limitations:**
- [LIMIT] **Only validates command name**, not arguments
  - `python3 script.py` [YES] validated
  - `python3 -c "import os; os.system('rm -rf /')"` [LIMIT] still dangerous
- [LIMIT] **Doesn't handle complex shell constructs**
  - `cmd1 | cmd2` - only validates first command
  - `cmd && cmd2` - only validates first command
  - `$(subcommand)` - subcommands not validated
- [LIMIT] **Path resolution issues**
  - `/usr/bin/python3` vs `python3` - different validation
  - Relative paths `./script.py` - not validated
- [LIMIT] **False sense of security**
  - Validates `python3` but `python3` can do anything
  - Validates `ls` but `ls` can read any accessible file

## Comparison: Current vs Proposed

### Security Comparison

| Aspect | Current Model | Proposed Model |
|--------|--------------|----------------|
| **Command Execution Control** | [NO] None | [YES] Command name validation |
| **Argument Validation** | [NO] None | [NO] None (same) |
| **Complex Shell Constructs** | [NO] Not handled | [NO] Not handled (same) |
| **Audit Trail** | [NO] Limited | [YES] All commands logged |
| **Approval Workflow** | [NO] None for commands | [YES] Can require approval |
| **Sandbox Protection** | [YES] Full | [YES] Full (same) |
| **False Security** | [YES] None | [LIMIT] May create false sense |

### Architecture Comparison

| Aspect | Current Model | Proposed Model |
|--------|--------------|----------------|
| **Simplicity** | [YES] Very simple | [LIMIT] More complex |
| **Performance** | [YES] Direct execution | [LIMIT] Validation overhead |
| **Separation of Concerns** | [YES] Clean (broker vs executor) | [LIMIT] Blurred (validation + execution) |
| **Agent Autonomy** | [YES] Full autonomy | [LIMIT] Requires validation |
| **Two-Step Process** | [YES] Single step | [LIMIT] May need two steps |

### Practical Comparison

| Use Case | Current Model | Proposed Model |
|----------|--------------|----------------|
| **Simple commands** (`ls`, `cat`) | [YES] Works | [YES] Works + validated |
| **Python scripts** | [YES] Works | [YES] Works + validated |
| **Pipelines** (`cmd1 \| cmd2`) | [YES] Works | [LIMIT] Only first cmd validated |
| **Complex scripts** | [YES] Works | [LIMIT] May not parse correctly |
| **Interactive tools** | [YES] Works | [YES] Works + validated |
| **Secret injection** | [YES] Secure | [YES] Secure (same) |

## Real-World Security Impact

### What Command Validation Actually Prevents

**[YES] Prevents:**
- Accidental execution of dangerous commands (`rm -rf /`)
- Execution of unauthorized tools (`nc`, `curl` to external hosts)
- Basic command whitelisting

**[NO] Doesn't Prevent:**
- Dangerous arguments: `python3 -c "dangerous_code()"`
- Script execution: `python3 malicious_script.py`
- Complex shell constructs: `cmd1 && dangerous_cmd`
- Path-based attacks: `/usr/bin/python3` vs `python3`

### The Real Question: Is Command Name Validation Enough?

**Arguments FOR:**
- [YES] Better than nothing - prevents obvious attacks
- [YES] Can whitelist safe commands (`ls`, `cat`, `grep`)
- [YES] Audit trail for compliance
- [YES] Approval workflow for sensitive commands

**Arguments AGAINST:**
- [LIMIT] False sense of security - validates `python3` but `python3` can do anything
- [LIMIT] Doesn't prevent the real threats (malicious scripts, dangerous args)
- [LIMIT] Adds complexity without proportional security benefit
- [LIMIT] May break legitimate use cases (complex scripts)

## Recommendation: Hybrid Approach

### Option: Selective Command Validation

Instead of validating ALL commands, validate only:
1. **Network commands** (`curl`, `wget`, `nc`, `ssh`)
2. **System modification commands** (`rm`, `chmod`, `sudo`)
3. **Package management** (already handled via `RequestPackage`)

**Implementation:**
```rust
// Only validate "dangerous" commands
const DANGEROUS_COMMANDS: &[&str] = &["curl", "wget", "nc", "rm", "chmod", "sudo"];

if DANGEROUS_COMMANDS.contains(&cmd_name) {
    validate_command(cmd_name)?;
}
// Otherwise, allow (sandbox provides protection)
```

### Why This Is Better

1. **Focused Security**: Only restricts actually dangerous commands
2. **Maintains Simplicity**: Doesn't validate every `ls` or `cat`
3. **Real Protection**: Prevents network access and system modification
4. **Less False Security**: Doesn't pretend to protect against script execution

## Final Verdict

### Would It Work? **YES**
- Technically feasible
- Can be implemented with `shlex` + policy engine
- Would provide command name validation

### Is It Good? **MIXED**

**Good For:**
- [YES] Compliance/audit requirements
- [YES] Preventing obvious dangerous commands
- [YES] Approval workflows for sensitive commands
- [YES] Network/system modification prevention (if selective)

**Not Good For:**
- [NO] Preventing malicious scripts (they'll use allowed interpreters)
- [NO] Argument validation (would need much more complex solution)
- [NO] Complex shell constructs (pipes, redirects, etc.)
- [NO] Maintaining simplicity of current model

### Recommendation

**If you need command validation:**
1. **Start with selective validation** - only dangerous commands
2. **Focus on network/system commands** - real security benefit
3. **Keep it optional** - don't break current workflows
4. **Document limitations** - don't create false sense of security

**If you don't need it:**
- Current model is simpler and cleaner
- Sandbox provides real protection
- Package/secret brokering is the real security boundary
- Command validation adds complexity without proportional benefit

## Alternative: Enhanced Audit Trail

Instead of command validation, consider:
- **Log all command executions** (via sandbox hooks or wrapper)
- **Post-execution analysis** (detect suspicious patterns)
- **Compliance reporting** (what commands were run, when)

This provides audit benefits without the complexity and false security of pre-execution validation.
