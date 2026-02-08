# OpenLeash - First Mission Walkthrough

This guide will take you from a fresh installation to running a sandboxed AI agent mission with OpenLeash.

## Step 1: Initialization

First, let's set up your local environment.

```bash
# Run the onboarding wizard
openleash init
```

This command creates a `~/.openleash` directory containing:
- `config.yaml`: Global settings for the daemon.
- `policies.yaml`: Rules for what your agent can access.
- `agent.sb`: A macOS sandbox profile tailored for your machine.

## Step 2: Configure a Strict Policy

Open `~/.openleash/policies.yaml`. By default, it allows everything. Let's make it strict to see Leash in action.

Replace the content with this:

```yaml
- id: "deny-all"
  name: "Deny All"
  resource_type: Package
  priority: 0
  allowed_patterns: [".*"]
  max_ttl_seconds: 0
  auto_approve: false
```

## Step 3: Start the Gatekeeper

Start the OpenLeash daemon in a separate terminal:

```bash
openleashd
```

## Step 4: The "Fail-Closed" Test

Try to install a package while inside the sandbox. It should fail because our policy is "deny-all".

```bash
# Run the openleash client inside the macOS sandbox
sandbox-exec -f ~/.openleash/agent.sb openleash request install --manager pip --package requests --scope /tmp/test-scope --reason "testing"
```

You should see a **Permission Denied** error. This confirms that your agent is properly secured.

## Step 5: Granting Permission

Update `~/.openleash/policies.yaml` to allow the `requests` library:

```yaml
- id: "allow-requests"
  name: "Allow Requests Library"
  resource_type: Package
  priority: 10
  allowed_patterns: ["^requests$"]
  max_ttl_seconds: 3600
  auto_approve: true

- id: "deny-all"
  ...
```

Restart the daemon (or wait for it to reload if implemented).

## Step 6: The Mission

Now, let's run a full mission.

1.  **Start a Task**:
    ```bash
    openleash task start --name "Web Scraping" --base-path /tmp/agent-work --ttl 3600
    # Copy the TASK_ID and SCOPE_PATH from the output
    ```

2.  **Install the Tool**:
    ```bash
    openleash request install --manager pip --package requests --task-id <TASK_ID>
    ```

3.  **Run the Agent**:
    ```bash
    # Brokered execution: the daemon runs the command and streams output
    openleash run --task-id <TASK_ID> --reason "running scraper" -- python my_scraper.py
    ```

    *Note: `openopenleash run` is different from `openopenleash exec`. `run` happens via the daemon (allowing the daemon to enforce strict command-level policies), whereas `exec` happens locally but with injected secrets.*

4.  **Cleanup**:
    ```bash
    openleash task end --task-id <TASK_ID>
    ```

## Step 7: Audit the Evidence

OpenLeash maintains a hash-chained ledger of every action. You can inspect this at any time.

```bash
# List the last 10 operations
openleash audit list --limit 10

# Verify the integrity of the ledger
openleash audit verify
```

Verification: `ls /tmp/agent-work` should show that the environment has been completely removed.

## Conclusion

You've successfully:
1.  Enforced a security boundary.
2.  Audited a request.
3.  Provided a scoped environment.
4.  Ensured clean resource teardown.

Your agent is now safe to work!
