# OpenClaw Direct CLI Integration

OpenLeash is designed to be exposed directly to AI agents as a set of **CLI Tools**. Instead of writing code to call OpenLeash, you describe the `openleash` commands to your LLM (using Tool Use / Function Calling) and allow the agent to invoke them directly.

## The OpenLeash Toolset

Expose these three commands to your agent's tool environment:

### 1. `openleash_task_start(name, ttl)`
**Description**: Call this at the start of a mission to create a secure, isolated environment.
**CLI Command**: `openleash task start --name "{{name}}" --ttl {{ttl}} --base-path /tmp/openleash-tasks`
**Agent Goal**: Establish a session ID and a workspace path.

### 2. `openleash_install(package, task_id)`
**Description**: Call this when you realize you are missing a dependency (e.g., `requests`, `pandas`).
**CLI Command**: `openleash request install --manager pip --package "{{package}}" --task-id {{task_id}} --reason "{{rationale}}"`
**Agent Goal**: Dynamically add capabilities to the current task.

### 3. `openleash_get_task_path(task_id)`
**Description**: Get the task environment PATH for direct command execution.
**CLI Command**: `openleash run --task-id {{task_id}}`
**Agent Goal**: Get PATH environment variable to execute commands directly. Use with: `eval $(openleash run --task-id {{task_id}})` then execute commands normally.

---

## Example LLM Prompt / Tool Definition

When configuring your agent, define the tool like this:

```json
{
  "name": "install_python_library",
  "description": "Installs a Python library into your secure task environment using OpenLeash.",
  "parameters": {
    "type": "object",
    "properties": {
      "package": { "type": "string", "description": "The name of the pip package (e.g., 'beautifulsoup4')" },
      "task_id": { "type": "string", "description": "The current OpenLeash Task UUID" },
      "rationale": { "type": "string", "description": "Why you need this library" }
    },
    "required": ["package", "task_id", "rationale"]
  },
  "implementation": "openleash request install --manager pip --package {{package}} --task-id {{task_id}} --reason {{rationale}}"
}
```

## The Agent Workflow (No SDK)

1.  **User**: "Scrape this website and save it to a CSV."
2.  **Agent**: *Checks environment, notices `pandas` is missing.*
3.  **Agent**: *Invokes Tool* → `openleash task start --name "Web Scrape"`
4.  **Agent**: *Invokes Tool* → `openleash request install --manager pip --package pandas --task-id <ID> --reason "CSV processing"`
5.  **Agent**: *Invokes Tool* → `eval $(openleash run --task-id <ID>)` then executes `python3 scrape.py` directly
6.  **Agent**: *Invokes Tool* → `openleash task end --task-id <ID>`

## Security Note
Because the agent is running inside a **macOS Sandbox** (defined in `agent.sb`), it cannot run `pip install` or access secrets directly. It **must** use the `openleash` CLI to request packages and secrets from the daemon. Once packages are installed, the agent executes commands directly in its sandbox using the PATH provided by `openleash run`. This ensures resource access is policy-checked and audited, while execution happens in the sandbox without privilege escalation.