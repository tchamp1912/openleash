# OpenClaw Integration Guide
## Solving Real Pain Points in AI Agent Operations

This guide shows how to integrate [Leash AI](https://github.com/openclaw/leash-ai) with the [OpenClaw AI harness](https://github.com/openclaw/openclaw) to solve specific operational challenges.

---

## Your Current Pain Points → Solutions

### 🔧 Problem 1: Manual Tool Installation

**Current State**:
```bash
# Agent needs jq, you manually:
brew install jq

# Agent needs kubectl, you manually:
brew install kubectl

# Repeat forever...
```

**With Leash AI**:
```python
# In OpenClaw, agent automatically requests:
async def run_task(self):
    # Agent realizes it needs jq
    try:
        result = subprocess.run(["jq", "--version"])
    except FileNotFoundError:
        # Request permission to install
        await self.leash_client.request_package(
            name="jq",
            rationale="Need jq to parse JSON from GitHub API response",
            temporary=True,
            ttl=3600  # Auto-remove after 1 hour
        )
        # Now available!
        result = subprocess.run(["jq", ...])
```

**Policy** (auto-approve common dev tools):
```yaml
- id: "dev-tools-auto"
  resource_type: "package"
  permission_level: "allow_auto"
  package_patterns:
    - "^(jq|yq|curl|wget|httpie)$"
  allow_temporary_only: true
  max_install_duration_seconds: 600
```

**Result**: Agent gets tools automatically, you don't need to be home. Tools auto-cleanup after use.

---

### 🔐 Problem 2: Token Blast Radius

**Current State**:
```python
# config.yaml - Full access token, forever
github_token: "ghp_xxxxxxxxxxxxxxxxxxxx"  # Can do EVERYTHING
aws_access_key: "AKIA..."  # Full AWS account access

# Agent has unlimited power
```

**With Leash AI**:
```python
# Agent requests scoped access
secret = await self.leash_client.request_secret(
    key="github/readonly-token",  # Scoped token, not full access
    rationale="List repositories for documentation task",
    ttl=1800  # 30 minutes only
)

# Use the token
gh = Github(secret)
repos = gh.get_user().get_repos()

# Token auto-expires after 30 minutes
# No revocation needed!
```

**Policy** (different tokens for different operations):
```yaml
# Read-only GitHub access (auto-approve)
- id: "github-readonly"
  resource_type: "secret"
  permission_level: "allow_auto"
  secret_patterns:
    - "github/readonly-.*"
  max_ttl_seconds: 3600

# Write access (requires approval)
- id: "github-write"
  resource_type: "secret"
  permission_level: "allow_with_approval"
  secret_patterns:
    - "github/write-.*"
  approvers:
    - "your-telegram-id"
  max_ttl_seconds: 1800
```

**Store tokens with different scopes**:
```bash
# Store in keychain (macOS)
leash secret store \
  --key "github/readonly-token" \
  --value "ghp_readonly_xxxx" \
  --type token

leash secret store \
  --key "github/write-token" \
  --value "ghp_fullaccess_xxxx" \
  --type token
```

**Result**: Agent only gets minimum necessary permissions, time-limited, auto-expiring.

---

### 📱 Problem 3: Remote Approval via Telegram

**Current State**:
```
Agent: "I need kubectl"
You: [At dinner, can't SSH home]
Agent: [Stuck forever]
```

**With Leash AI + Telegram Bot**:

```python
# telegram_approval_bot.py
from telegram import Update, InlineKeyboardButton, InlineKeyboardMarkup
from telegram.ext import Application, CommandHandler, CallbackQueryHandler

class TelegramApprovalBot:
    """Telegram bot for remote approvals."""
    
    async def send_approval_request(self, request):
        """Send approval request to Telegram."""
        text = f"""
🔐 **Permission Request**

**Agent**: {request.instance_id}
**Resource**: {request.resource_type}:{request.resource_id}
**Rationale**: {request.rationale}

**Requested**: {request.created_at}
**Expires**: {request.expires_at}
"""
        
        keyboard = [
            [
                InlineKeyboardButton("✅ Approve", callback_data=f"approve:{request.request_id}"),
                InlineKeyboardButton("❌ Deny", callback_data=f"deny:{request.request_id}"),
            ],
            [
                InlineKeyboardButton("📊 View Details", callback_data=f"details:{request.request_id}"),
            ]
        ]
        
        await self.bot.send_message(
            chat_id=self.your_chat_id,
            text=text,
            reply_markup=InlineKeyboardMarkup(keyboard),
            parse_mode="Markdown"
        )
    
    async def handle_callback(self, update: Update, context):
        """Handle button clicks."""
        query = update.callback_query
        action, request_id = query.data.split(":")
        
        if action == "approve":
            await self.perm_daemon.approve_request(request_id)
            await query.answer("✅ Approved!")
            await query.edit_message_text(
                text=f"{query.message.text}\n\n✅ **APPROVED** by you via Telegram"
            )
        
        elif action == "deny":
            await self.perm_daemon.deny_request(request_id)
            await query.answer("❌ Denied")
            await query.edit_message_text(
                text=f"{query.message.text}\n\n❌ **DENIED** by you via Telegram"
            )
```

**Integration with Leash AI**:
```python
# In Leash AI daemon config
approval_backends:
  - type: telegram
    bot_token: ${TELEGRAM_BOT_TOKEN}
    chat_id: ${YOUR_TELEGRAM_CHAT_ID}
    timeout: 900  # 15 minutes
```

**Result**: Get push notification on Telegram, approve with one tap, from anywhere.

---

### 🔑 Problem 4: Plaintext API Keys in Config

**Current State**:
```yaml
# config.yaml - Agent can read this!
anthropic_api_key: "sk-ant-xxxxxxxxxxxx"
openai_api_key: "sk-xxxxxxxxxxxxxxxx"
github_token: "ghp_xxxxxxxxxxxx"
```

**With Leash AI**:
```yaml
# config.yaml - No secrets!
anthropic_api_key: "use-leash-ai"
openai_api_key: "use-leash-ai"
github_token: "use-leash-ai"

# Enable permission client
permissions:
  enabled: true
  daemon_url: "http://localhost:8765"
  instance_id: "openclaw-main"
```

**Agent wrapper** (in OpenClaw):
```python
class SecureAnthropicClient:
    """Wrapper that requests API key on demand."""
    
    def __init__(self, leash_client):
        self.leash_client = leash_client
        self._api_key = None
        self._key_expires = None
    
    async def _ensure_key(self):
        """Get API key if needed."""
        if self._api_key is None or datetime.now() > self._key_expires:
            self._api_key = await self.leash_client.request_secret(
                key="anthropic/api-key",
                rationale="Make API call for user task",
                ttl=3600
            )
            self._key_expires = datetime.now() + timedelta(seconds=3600)
    
    async def messages_create(self, **kwargs):
        """Create message with automatic key retrieval."""
        await self._ensure_key()
        client = anthropic.AsyncAnthropic(api_key=self._api_key)
        return await client.messages.create(**kwargs)

# Usage in OpenClaw
anthropic = SecureAnthropicClient(leash_client)
response = await anthropic.messages_create(...)
```

**Store keys in keychain**:
```bash
# One-time setup (keys never in config files)
security add-generic-password \
  -s leash-ai \
  -a "anthropic/api-key" \
  -w "sk-ant-xxxxxxxxxxxx"
  
security add-generic-password \
  -s leash-ai \
  -a "openai/api-key" \
  -w "sk-xxxxxxxxxxxxxxxx"
```

**Result**: API keys never in config, agent gets them on-demand, time-limited.

---

### 🛡️ Problem 5: No Automatic Revocation

**Current State**:
```bash
# Give agent GitHub token
export GITHUB_TOKEN="ghp_xxxx"

# [2 weeks later, you forget it's still active]
# Agent still has access forever
```

**With Leash AI**:
```python
# All access is time-limited by default
secret = await leash_client.request_secret(
    key="github/token",
    rationale="Deploy v2.1.0",
    ttl=1800  # 30 minutes
)

# After 30 minutes, token is INVALID
# Agent must request again with new rationale
```

**Emergency revocation**:
```bash
# From your phone via SSH + Telegram
leash revoke --instance openclaw-main --resource "github/*"

# Or revoke everything
leash revoke --instance openclaw-main --all
```

**Result**: All access auto-expires, emergency revocation available.

---

### 🔍 Problem 6: Context Window Leakage Risk

**Current State**:
```python
# Agent execution
result = subprocess.run(["aws", "s3", "ls"], capture_output=True)
print(result.stdout)  # Bucket names in context window
print(result.stderr)  # Error messages might have sensitive data

# Agent can see all this and potentially leak it
```

**With Leash AI** (filtered execution):
```python
# Execute command with output filtering
result = await leash_client.execute_command(
    command="aws",
    args=["s3", "ls"],
    rationale="List S3 buckets for backup task",
    filter_output=True,  # Redact sensitive patterns
    filter_patterns=[
        r"arn:aws:.*",  # ARNs
        r"bucket://.*",  # Bucket URLs
        r"\d{12}",  # Account IDs
    ]
)

# result.stdout has sensitive data redacted
# Original stored in audit log (not in context)
```

**Audit trail** (not in context window):
```bash
# Review what really happened (outside agent's view)
leash audit list --instance openclaw-main --today

# Output:
# 14:23:15 | openclaw-main | EXECUTE | aws s3 ls | SUCCESS
#   Output: [32 buckets listed - see secure audit log]
#   Rationale: "List S3 buckets for backup task"
```

**Result**: Sensitive output doesn't leak into context window, stored securely for audit.

---

### 🔌 Problem 7: MCP Token Exposure

**Current State**:
```yaml
# MCP server config
mcp_servers:
  github:
    token: "ghp_xxxxxxxxxxxx"  # Real token
  slack:
    token: "xoxb-xxxxxxxxxxxx"  # Real token
  
# Agent has access to all MCP configs
```

**With Leash AI** (MCP Proxy):
```python
# mcp_proxy.py
class SecureMCPProxy:
    """Proxy MCP servers with token injection."""
    
    def __init__(self, leash_client):
        self.leash_client = leash_client
        self.servers = {}
    
    async def call_mcp_server(self, server: str, method: str, params: dict):
        """Proxy MCP call with token injection."""
        # Get token from permission system
        token = await self.leash_client.request_secret(
            key=f"mcp/{server}/token",
            rationale=f"Call {server}.{method} for user task",
            ttl=300  # 5 minutes
        )
        
        # Initialize MCP server with token
        mcp_client = MCPClient(server_url=self.servers[server], token=token)
        
        # Make actual call
        return await mcp_client.call(method, params)

# Usage in OpenClaw
mcp = SecureMCPProxy(leash_client)
result = await mcp.call_mcp_server("github", "list_repos", {})
# Agent never sees the real token!
```

**MCP config** (no tokens):
```yaml
# mcp_servers.yaml - Agent sees this
mcp_servers:
  github:
    url: "http://localhost:8001"  # Local proxy
  slack:
    url: "http://localhost:8002"
  
# Real tokens stored in keychain
```

**Result**: Agent uses MCP servers, never sees real tokens.

---

## Complete Integration Example

### Step 1: Install & Setup

```bash
# Install Leash AI
pip install leash-ai

# Start daemon
leash start

# Store secrets (one-time)
security add-generic-password -s leash-ai -a "anthropic/api-key" -w "$ANTHROPIC_KEY"
security add-generic-password -s leash-ai -a "openai/api-key" -w "$OPENAI_KEY"
security add-generic-password -s leash-ai -a "github/token" -w "$GITHUB_TOKEN"
```

### Step 2: Load Policies

```yaml
# openclaw-policies.yaml
version: "1.0"
default_permission: "deny"

policies:
  # API keys - auto approve with short TTL
  - id: "ai-api-keys"
    resource_type: "secret"
    permission_level: "allow_auto"
    secret_patterns:
      - "anthropic/.*"
      - "openai/.*"
    max_ttl_seconds: 3600
    auto_approve_patterns:
      - ".*API call for user task.*"
  
  # GitHub read - auto approve
  - id: "github-read"
    resource_type: "secret"
    permission_level: "allow_auto"
    secret_patterns:
      - "github/readonly-.*"
    max_ttl_seconds: 1800
  
  # GitHub write - requires approval
  - id: "github-write"
    resource_type: "secret"
    permission_level: "allow_with_approval"
    secret_patterns:
      - "github/token"
    approvers:
      - "telegram:@yourusername"
    max_ttl_seconds: 900
  
  # Dev tools - auto install
  - id: "dev-tools"
    resource_type: "package"
    permission_level: "allow_auto"
    package_patterns:
      - "^(jq|yq|curl|wget|httpie|gh)$"
    allow_temporary_only: true
    max_install_duration_seconds: 600
  
  # Cloud CLIs - require approval
  - id: "cloud-cli"
    resource_type: "package"
    permission_level: "allow_with_approval"
    package_patterns:
      - "awscli"
      - "kubectl"
      - "gcloud"
    approvers:
      - "telegram:@yourusername"
  
  # Safe commands - auto approve
  - id: "readonly-commands"
    resource_type: "cli_command"
    permission_level: "allow_auto"
    command_patterns:
      - "^(ls|cat|grep|find|git)$"
    denied_args:
      - "--delete"
      - "-rf"
  
  # Dangerous commands - require approval
  - id: "sudo-commands"
    resource_type: "cli_command"
    permission_level: "allow_with_approval"
    allow_sudo: true
    approvers:
      - "telegram:@yourusername"
```

```bash
leash policy add openclaw-policies.yaml
```

### Step 3: Modify OpenClaw Integration

```python
# openclaw/core/permissions.py
from leash_ai import LeashClient

class OpenClawPermissions:
    """Permission integration for OpenClaw."""
    
    def __init__(self, instance_id: str):
        self.client = LeashClient(
            instance_id=instance_id,
            daemon_url="http://localhost:8765"
        )
        self._secret_cache = {}
    
    async def get_api_key(self, service: str, rationale: str) -> str:
        """Get API key with automatic request."""
        cache_key = f"{service}/api-key"
        
        # Check cache
        if cache_key in self._secret_cache:
            cached = self._secret_cache[cache_key]
            if cached['expires'] > datetime.now():
                return cached['value']
        
        # Request from permission system
        secret = await self.client.request_secret(
            key=cache_key,
            rationale=rationale,
            ttl=3600
        )
        
        # Cache
        self._secret_cache[cache_key] = {
            'value': secret,
            'expires': datetime.now() + timedelta(seconds=3600)
        }
        
        return secret
    
    async def ensure_tool(self, tool: str, rationale: str):
        """Ensure tool is installed."""
        # Check if available
        if shutil.which(tool):
            return
        
        # Request installation
        await self.client.request_package(
            name=tool,
            rationale=rationale,
            temporary=True,
            ttl=3600
        )
    
    async def execute_command(
        self,
        command: str,
        args: list,
        rationale: str,
        sudo: bool = False
    ):
        """Execute command with permission check."""
        result = await self.client.execute_command(
            command=command,
            args=args,
            rationale=rationale,
            sudo=sudo,
            timeout=300
        )
        return result

# Usage in OpenClaw
perms = OpenClawPermissions(instance_id="openclaw-main")

# Get API key
api_key = await perms.get_api_key("anthropic", "User asked me to summarize article")
client = anthropic.AsyncAnthropic(api_key=api_key)

# Ensure tool
await perms.ensure_tool("jq", "Parse JSON from GitHub API")

# Execute command
result = await perms.execute_command(
    command="git",
    args=["status"],
    rationale="Check git status before committing changes"
)
```

### Step 4: Setup Telegram Approval Bot

```python
# telegram_bot.py
import asyncio
from telegram import Update, InlineKeyboardButton, InlineKeyboardMarkup
from telegram.ext import Application, CallbackQueryHandler
import httpx

class ApprovalBot:
    def __init__(self, bot_token: str, perm_daemon_url: str):
        self.app = Application.builder().token(bot_token).build()
        self.daemon_url = perm_daemon_url
        self.app.add_handler(CallbackQueryHandler(self.handle_button))
        
        # Poll for pending requests
        asyncio.create_task(self.poll_pending())
    
    async def poll_pending(self):
        """Poll for pending approval requests."""
        while True:
            async with httpx.AsyncClient() as client:
                response = await client.get(
                    f"{self.daemon_url}/api/v1/requests/pending"
                )
                requests = response.json()
                
                for req in requests:
                    await self.send_approval_request(req)
            
            await asyncio.sleep(10)
    
    async def send_approval_request(self, request):
        """Send approval notification."""
        text = f"""
🔐 **Permission Request**

**Agent**: `{request['instance_id']}`
**Resource**: `{request['resource_type']}:{request['resource_id']}`

**Rationale**:
_{request['rationale']}_

**Requested**: {request['created_at']}
**Policy**: {request['policy_matched']}
"""
        
        keyboard = [
            [
                InlineKeyboardButton("✅ Approve", callback_data=f"approve:{request['request_id']}"),
                InlineKeyboardButton("❌ Deny", callback_data=f"deny:{request['request_id']}"),
            ]
        ]
        
        await self.app.bot.send_message(
            chat_id=YOUR_CHAT_ID,
            text=text,
            reply_markup=InlineKeyboardMarkup(keyboard),
            parse_mode="Markdown"
        )
    
    async def handle_button(self, update: Update, context):
        """Handle approval/denial."""
        query = update.callback_query
        action, request_id = query.data.split(":")
        
        async with httpx.AsyncClient() as client:
            if action == "approve":
                await client.post(
                    f"{self.daemon_url}/api/v1/requests/{request_id}/approve"
                )
                await query.answer("✅ Approved!")
            else:
                await client.post(
                    f"{self.daemon_url}/api/v1/requests/{request_id}/deny"
                )
                await query.answer("❌ Denied")
        
        await query.edit_message_text(
            text=f"{query.message.text}\n\n**Decision**: {action.upper()}"
        )
    
    def run(self):
        """Start bot."""
        self.app.run_polling()

if __name__ == "__main__":
    bot = ApprovalBot(
        bot_token=os.getenv("TELEGRAM_BOT_TOKEN"),
        perm_daemon_url="http://localhost:8765"
    )
    bot.run()
```

---

## Benefits Summary

| Problem | Before | After |
|---------|--------|-------|
| **Tool Installation** | Manual `brew install` | Auto-request, auto-install, auto-remove |
| **Token Exposure** | Plaintext in config | Keychain, time-limited, scoped |
| **Remote Approval** | SSH home or wait | Telegram push notification |
| **Revocation** | Manual, often forgotten | Automatic expiration |
| **Context Leakage** | All output visible | Filtered, audit trail separate |
| **MCP Tokens** | Agent sees real tokens | Proxy with token injection |
| **Blast Radius** | Full access forever | Minimum scope, short-lived |
| **Auditability** | Ad-hoc logs | Complete, queryable audit trail |

---

## Next Steps

1. **Install Leash AI**: `pip install leash-ai`
2. **Start daemon**: `leash start`
3. **Load policies**: `leash policy add openclaw-policies.yaml`
4. **Setup Telegram bot**: `python telegram_bot.py`
5. **Integrate with OpenClaw**: Add `OpenClawPermissions` class
6. **Test**: Request a secret/package, approve via Telegram

---

## Additional Resources

- [Leash AI Documentation](../README.md)
- [Security Design](../SECURITY_DESIGN.md)
- [Threat Model](../THREAT_MODEL.md)
- [Extensibility Guide](../EXTENSIBILITY.md)

**Questions?** Open an issue in [Leash AI](https://github.com/openclaw/leash-ai) or [OpenClaw](https://github.com/openclaw/openclaw) repos.
