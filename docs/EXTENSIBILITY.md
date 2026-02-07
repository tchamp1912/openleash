# Extensibility & Plugin Architecture

## Overview

Leash AI is designed to be **highly extensible** through a well-defined plugin architecture. This document describes how to extend the system with custom backends, policies, and integrations.

## Design Principles

### 1. Interface Segregation
Each backend type has a focused, minimal interface. Implement only what you need.

### 2. Dependency Injection
All backends are injectable, making testing and swapping implementations easy.

### 3. Discovery & Registration
Plugins auto-discover through entry points (setuptools) or explicit registration.

### 4. Backward Compatibility
Plugin APIs follow semantic versioning. Deprecations announced 2 versions ahead.

---

## Extension Points

### 1. Secret Backends

**Purpose**: Add support for new secret storage systems.

**Interface**: `SecretBackend` (ABC)

**Examples**:
- Linux Secret Service (GNOME Keyring, KWallet)
- HashiCorp Vault
- AWS Secrets Manager
- Azure Key Vault
- 1Password CLI
- Custom encrypted file format

**How to Implement**:

```python
# my_plugin/backends/secrets/custom_vault.py

from leash_ai.backends.secrets import SecretBackend, Secret, SecretType
from typing import Optional, List, Dict, Any

class CustomVaultBackend(SecretBackend):
    """Custom Vault implementation."""
    
    def __init__(self, vault_url: str, token: str):
        self.vault_url = vault_url
        self.token = token
        self.client = None
    
    async def initialize(self) -> None:
        """Connect to vault."""
        self.client = CustomVaultClient(
            url=self.vault_url,
            token=self.token,
        )
        await self.client.connect()
    
    async def store(
        self,
        key: str,
        value: str,
        secret_type: SecretType = SecretType.GENERIC,
        metadata: Optional[Dict[str, Any]] = None,
        tags: Optional[Dict[str, str]] = None,
        ttl: Optional[int] = None,
    ) -> None:
        """Store secret in custom vault."""
        await self.client.put(
            path=f"openclaw/{key}",
            data={
                "value": value,
                "type": secret_type.value,
                "metadata": metadata or {},
                "tags": tags or {},
            },
            ttl=ttl,
        )
    
    async def retrieve(self, key: str) -> Secret:
        """Retrieve secret from custom vault."""
        data = await self.client.get(f"openclaw/{key}")
        
        return Secret(
            key=key,
            value=data["value"],
            secret_type=SecretType(data["type"]),
            metadata=data.get("metadata", {}),
            created_at=parse_iso(data["created_at"]),
            updated_at=parse_iso(data["updated_at"]),
            expires_at=parse_iso(data["expires_at"]) if data.get("expires_at") else None,
            tags=data.get("tags"),
        )
    
    # Implement remaining abstract methods...
```

**Register Plugin**:

```python
# setup.py or pyproject.toml

[project.entry-points."leash_ai.secret_backends"]
custom_vault = "my_plugin.backends.secrets:CustomVaultBackend"
```

**Use Plugin**:

```yaml
# config.yaml
backends:
  secrets:
    type: custom_vault
    vault_url: https://vault.company.com
    token: ${VAULT_TOKEN}
```

---

### 2. Package Manager Backends

**Purpose**: Add support for new package managers.

**Interface**: `PackageBackend` (ABC)

**Examples**:
- APT (Debian/Ubuntu)
- DNF/YUM (RedHat/Fedora)
- Snap (Universal Linux)
- Chocolatey (Windows)
- pip (Python packages)
- npm (Node packages)

**How to Implement**:

```python
# my_plugin/backends/package/apt.py

from leash_ai.backends.package import PackageBackend, Package, PackageStatus
import subprocess
from typing import Optional, List, Dict, Any

class APTBackend(PackageBackend):
    """APT package manager backend."""
    
    async def initialize(self) -> None:
        """Check APT is available."""
        result = subprocess.run(
            ["apt-get", "--version"],
            capture_output=True,
        )
        if result.returncode != 0:
            raise PackageManagerNotAvailableError("APT not available")
    
    async def is_available(self) -> bool:
        """Check if APT is available."""
        try:
            subprocess.run(
                ["apt-get", "--version"],
                capture_output=True,
                timeout=5,
            )
            return True
        except (FileNotFoundError, subprocess.TimeoutExpired):
            return False
    
    async def install(
        self,
        package_name: str,
        version: Optional[str] = None,
        timeout: int = 300,
    ) -> Package:
        """Install package via APT."""
        cmd = ["sudo", "apt-get", "install", "-y", package_name]
        if version:
            cmd[-1] = f"{package_name}={version}"
        
        # Execute with timeout
        process = await asyncio.create_subprocess_exec(
            *cmd,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        
        try:
            stdout, stderr = await asyncio.wait_for(
                process.communicate(),
                timeout=timeout,
            )
        except asyncio.TimeoutError:
            process.kill()
            raise PackageInstallError(f"Installation timed out after {timeout}s")
        
        if process.returncode != 0:
            raise PackageInstallError(f"APT install failed: {stderr.decode()}")
        
        # Return package info
        return Package(
            name=package_name,
            version=version or await self._get_installed_version(package_name),
            manager="apt",
            status=PackageStatus.INSTALLED,
            installed_at=datetime.utcnow(),
            binaries=await self.get_binaries(package_name),
            metadata={"output": stdout.decode()},
        )
    
    # Implement remaining abstract methods...
```

---

### 3. CLI Execution Backends

**Purpose**: Add custom command execution environments.

**Interface**: `CLIBackend` (ABC)

**Examples**:
- Docker container execution
- Kubernetes pod execution
- SSH remote execution
- Windows PowerShell
- Custom sandboxes

**How to Implement**:

```python
# my_plugin/backends/cli/docker.py

from leash_ai.backends.cli import CLIBackend, CommandRequest, CommandResult
import docker

class DockerCLIBackend(CLIBackend):
    """Execute commands in Docker containers."""
    
    def __init__(self, image: str = "ubuntu:22.04"):
        self.image = image
        self.client = None
    
    async def initialize(self) -> None:
        """Connect to Docker daemon."""
        self.client = docker.from_env()
    
    async def execute(
        self,
        request: CommandRequest,
        capture_output: bool = True,
    ) -> CommandResult:
        """Execute command in Docker container."""
        started_at = datetime.utcnow()
        
        # Validate command
        if not await self.is_command_available(request.command):
            raise CommandNotFoundError(f"Command not found: {request.command}")
        
        # Build command
        cmd = [request.command] + request.args
        
        # Run in container
        try:
            container = self.client.containers.run(
                self.image,
                command=cmd,
                environment=request.env or {},
                working_dir=request.working_dir or "/workspace",
                remove=True,
                detach=False,
                stdout=True,
                stderr=True,
            )
            
            # Parse output
            stdout = container.decode()
            stderr = ""
            exit_code = 0
            status = CommandStatus.SUCCESS
            
        except docker.errors.ContainerError as e:
            stdout = e.stdout.decode() if e.stdout else ""
            stderr = e.stderr.decode() if e.stderr else ""
            exit_code = e.exit_status
            status = CommandStatus.FAILED
        
        completed_at = datetime.utcnow()
        duration_ms = int((completed_at - started_at).total_seconds() * 1000)
        
        return CommandResult(
            request=request,
            status=status,
            exit_code=exit_code,
            stdout=stdout,
            stderr=stderr,
            started_at=started_at,
            completed_at=completed_at,
            duration_ms=duration_ms,
        )
    
    # Implement remaining abstract methods...
```

---

### 4. Custom Policy Types

**Purpose**: Add domain-specific policy types.

**Base Class**: `PermissionPolicy`

**Examples**:
- Network access policies (allowed domains, ports)
- File access policies (allowed paths, operations)
- Database query policies (read-only, specific tables)
- API endpoint policies (allowed endpoints, rate limits)

**How to Implement**:

```python
# my_plugin/policies/network.py

from leash_ai.policies.models import PermissionPolicy, ResourceType
from dataclasses import dataclass, field
from typing import List, Optional
import re

@dataclass
class NetworkPolicy(PermissionPolicy):
    """Policy for network access control."""
    
    allowed_domains: List[str] = field(default_factory=list)  # e.g., ["*.github.com", "api.openai.com"]
    denied_domains: List[str] = field(default_factory=list)   # e.g., ["*.internal.company.com"]
    allowed_ports: Optional[List[int]] = None  # None = all ports
    allowed_protocols: List[str] = field(default_factory=lambda: ["https", "http"])
    max_connections: int = 10
    max_bandwidth_mbps: Optional[float] = None
    
    def __post_init__(self):
        self.resource_type = ResourceType.NETWORK
    
    def matches_request(self, domain: str, port: int, protocol: str) -> bool:
        """Check if network request matches this policy."""
        # Check protocol
        if protocol not in self.allowed_protocols:
            return False
        
        # Check port
        if self.allowed_ports is not None and port not in self.allowed_ports:
            return False
        
        # Check denied domains first
        for pattern in self.denied_domains:
            if self._domain_matches(domain, pattern):
                return False
        
        # Check allowed domains
        if not self.allowed_domains:
            return True  # No restrictions
        
        return any(
            self._domain_matches(domain, pattern)
            for pattern in self.allowed_domains
        )
    
    def _domain_matches(self, domain: str, pattern: str) -> bool:
        """Check if domain matches pattern (supports wildcards)."""
        # Convert wildcard pattern to regex
        regex_pattern = pattern.replace(".", r"\.").replace("*", r"[^.]+")
        return bool(re.match(f"^{regex_pattern}$", domain))
```

**Use Custom Policy**:

```yaml
# policies/network-policies.yaml
policies:
  - id: "network-github-allow"
    name: "Allow GitHub Access"
    resource_type: "network"
    permission_level: "allow_auto"
    allowed_domains:
      - "*.github.com"
      - "github.com"
    allowed_protocols:
      - "https"
    allowed_ports:
      - 443
```

---

### 5. Approval Workflow Backends

**Purpose**: Integrate with custom approval systems.

**Interface**: `ApprovalBackend` (ABC)

**Examples**:
- Slack approval bot
- Email approval workflow
- PagerDuty integration
- ServiceNow integration
- Custom web UI

**How to Implement**:

```python
# my_plugin/approval/slack.py

from leash_ai.approval import ApprovalBackend, ApprovalRequest, ApprovalResponse
import slack_sdk

class SlackApprovalBackend(ApprovalBackend):
    """Slack-based approval workflow."""
    
    def __init__(self, bot_token: str, channel: str):
        self.bot_token = bot_token
        self.channel = channel
        self.client = None
    
    async def initialize(self) -> None:
        """Initialize Slack client."""
        self.client = slack_sdk.WebClient(token=self.bot_token)
    
    async def request_approval(
        self,
        request: ApprovalRequest,
    ) -> str:
        """Send approval request to Slack."""
        # Create interactive message
        blocks = [
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": f"*Permission Request*\n"
                            f"Instance: `{request.instance_id}`\n"
                            f"Resource: `{request.resource_type}:{request.resource_id}`\n"
                            f"Rationale: _{request.rationale}_"
                }
            },
            {
                "type": "actions",
                "elements": [
                    {
                        "type": "button",
                        "text": {"type": "plain_text", "text": "Approve"},
                        "style": "primary",
                        "value": f"approve:{request.request_id}",
                        "action_id": "approve_request"
                    },
                    {
                        "type": "button",
                        "text": {"type": "plain_text", "text": "Deny"},
                        "style": "danger",
                        "value": f"deny:{request.request_id}",
                        "action_id": "deny_request"
                    }
                ]
            }
        ]
        
        response = await self.client.chat_postMessage(
            channel=self.channel,
            blocks=blocks,
            text=f"Permission request from {request.instance_id}",
        )
        
        return response["ts"]  # Message timestamp as approval ID
    
    async def get_approval_status(
        self,
        approval_id: str,
    ) -> Optional[ApprovalResponse]:
        """Check if approval has been granted/denied."""
        # Check message reactions or button clicks
        # Implementation depends on Slack event handling
        ...
    
    async def cancel_approval(self, approval_id: str) -> None:
        """Cancel pending approval request."""
        # Delete or update Slack message
        ...
```

---

### 6. Audit Log Backends

**Purpose**: Send audit logs to custom destinations.

**Interface**: `AuditBackend` (ABC)

**Examples**:
- Splunk
- Elasticsearch
- AWS CloudWatch Logs
- Datadog
- Custom SIEM

**How to Implement**:

```python
# my_plugin/audit/elasticsearch.py

from leash_ai.audit import AuditBackend, AuditEvent
from elasticsearch import AsyncElasticsearch
from datetime import datetime

class ElasticsearchAuditBackend(AuditBackend):
    """Send audit logs to Elasticsearch."""
    
    def __init__(self, hosts: List[str], index: str = "openclaw-audit"):
        self.hosts = hosts
        self.index = index
        self.client = None
    
    async def initialize(self) -> None:
        """Connect to Elasticsearch."""
        self.client = AsyncElasticsearch(hosts=self.hosts)
    
    async def log_event(self, event: AuditEvent) -> None:
        """Log event to Elasticsearch."""
        doc = {
            "timestamp": event.timestamp.isoformat(),
            "event_type": event.event_type,
            "instance_id": event.instance_id,
            "resource_type": event.resource_type,
            "resource_id": event.resource_id,
            "action": event.action,
            "status": event.status,
            "rationale": event.rationale,
            "metadata": event.metadata,
        }
        
        await self.client.index(
            index=self.index,
            document=doc,
        )
    
    async def query_events(
        self,
        start_time: Optional[datetime] = None,
        end_time: Optional[datetime] = None,
        instance_id: Optional[str] = None,
        event_type: Optional[str] = None,
        limit: int = 100,
    ) -> List[AuditEvent]:
        """Query events from Elasticsearch."""
        query = {"bool": {"must": []}}
        
        if start_time or end_time:
            time_range = {}
            if start_time:
                time_range["gte"] = start_time.isoformat()
            if end_time:
                time_range["lte"] = end_time.isoformat()
            query["bool"]["must"].append({
                "range": {"timestamp": time_range}
            })
        
        if instance_id:
            query["bool"]["must"].append({
                "term": {"instance_id": instance_id}
            })
        
        if event_type:
            query["bool"]["must"].append({
                "term": {"event_type": event_type}
            })
        
        response = await self.client.search(
            index=self.index,
            query=query,
            size=limit,
            sort=[{"timestamp": {"order": "desc"}}],
        )
        
        return [
            self._parse_event(hit["_source"])
            for hit in response["hits"]["hits"]
        ]
```

---

## Plugin Discovery & Registration

### Method 1: Entry Points (Recommended)

```toml
# pyproject.toml for plugin package

[project.entry-points."leash_ai.secret_backends"]
my_vault = "my_plugin.backends.secrets:MyVaultBackend"

[project.entry-points."leash_ai.package_backends"]
apt = "my_plugin.backends.package:APTBackend"

[project.entry-points."leash_ai.cli_backends"]
docker = "my_plugin.backends.cli:DockerCLIBackend"

[project.entry-points."leash_ai.approval_backends"]
slack = "my_plugin.approval:SlackApprovalBackend"

[project.entry-points."leash_ai.audit_backends"]
elasticsearch = "my_plugin.audit:ElasticsearchAuditBackend"
```

Plugins are automatically discovered:

```python
# In leash_ai core
import importlib.metadata

def discover_plugins(group: str) -> Dict[str, Type]:
    """Discover plugins via entry points."""
    plugins = {}
    for entry_point in importlib.metadata.entry_points(group=group):
        plugins[entry_point.name] = entry_point.load()
    return plugins

# Usage
secret_backends = discover_plugins("leash_ai.secret_backends")
vault_backend = secret_backends["my_vault"](url="...", token="...")
```

### Method 2: Explicit Registration

```python
# In plugin code
from leash_ai.registry import register_backend

@register_backend("secret", "my_vault")
class MyVaultBackend(SecretBackend):
    ...

# Or programmatically
from leash_ai.registry import BackendRegistry

registry = BackendRegistry()
registry.register_secret_backend("my_vault", MyVaultBackend)
```

---

## Testing Plugins

### Unit Tests

```python
# tests/test_my_vault_backend.py

import pytest
from my_plugin.backends.secrets import MyVaultBackend

@pytest.fixture
async def backend():
    """Create backend instance."""
    backend = MyVaultBackend(url="http://localhost:8200", token="test-token")
    await backend.initialize()
    yield backend
    await backend.close()

@pytest.mark.asyncio
async def test_store_and_retrieve(backend):
    """Test storing and retrieving secrets."""
    await backend.store(
        key="test/secret",
        value="secret-value",
        secret_type=SecretType.API_KEY,
    )
    
    secret = await backend.retrieve("test/secret")
    assert secret.value == "secret-value"
    assert secret.secret_type == SecretType.API_KEY

@pytest.mark.asyncio
async def test_secret_expiration(backend):
    """Test secrets expire correctly."""
    await backend.store(
        key="test/temp",
        value="temp-value",
        ttl=1,  # 1 second
    )
    
    await asyncio.sleep(2)
    
    with pytest.raises(SecretNotFoundError):
        await backend.retrieve("test/temp")
```

### Integration Tests

```python
# tests/integration/test_vault_integration.py

@pytest.mark.integration
async def test_end_to_end_secret_access():
    """Test full workflow with custom backend."""
    # Setup: Configure daemon with custom backend
    daemon = PermissionDaemon(config={
        "backends": {
            "secrets": {
                "type": "my_vault",
                "url": "http://localhost:8200",
                "token": os.getenv("VAULT_TOKEN"),
            }
        }
    })
    
    await daemon.start()
    
    # Test: OpenClaw requests secret via Leash AI client
    client = LeashClient(instance_id="test-instance")
    secret = await client.request_secret(
        key="test/api-key",
        rationale="Integration test",
    )
    
    assert secret is not None
    
    await daemon.stop()
```

---

## Plugin Best Practices

### 1. Error Handling

```python
class MyBackend(SecretBackend):
    async def retrieve(self, key: str) -> Secret:
        try:
            value = await self._fetch_from_backend(key)
        except ConnectionError as e:
            # Re-raise as backend-specific error
            raise SecretBackendError(f"Backend connection failed: {e}")
        except KeyError:
            # Use standard exception types
            raise SecretNotFoundError(f"Secret not found: {key}")
        except Exception as e:
            # Log unexpected errors
            logger.error(f"Unexpected error retrieving secret: {e}", exc_info=True)
            raise SecretBackendError("Internal backend error")
```

### 2. Configuration Validation

```python
from pydantic import BaseModel, validator

class MyVaultConfig(BaseModel):
    """Configuration for My Vault backend."""
    url: str
    token: str
    timeout: int = 30
    max_retries: int = 3
    
    @validator("url")
    def validate_url(cls, v):
        if not v.startswith("http"):
            raise ValueError("URL must start with http:// or https://")
        return v
    
    @validator("timeout")
    def validate_timeout(cls, v):
        if v < 1 or v > 300:
            raise ValueError("Timeout must be between 1 and 300 seconds")
        return v

class MyVaultBackend(SecretBackend):
    def __init__(self, config: MyVaultConfig):
        self.config = config
```

### 3. Resource Cleanup

```python
class MyBackend(SecretBackend):
    def __init__(self):
        self.client = None
        self._initialized = False
    
    async def initialize(self) -> None:
        """Initialize backend."""
        self.client = await create_client()
        self._initialized = True
    
    async def close(self) -> None:
        """Clean up resources."""
        if self.client:
            await self.client.close()
            self.client = None
        self._initialized = False
    
    async def __aenter__(self):
        """Context manager support."""
        await self.initialize()
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Context manager cleanup."""
        await self.close()
```

### 4. Logging

```python
import logging

logger = logging.getLogger(__name__)

class MyBackend(SecretBackend):
    async def retrieve(self, key: str) -> Secret:
        logger.debug(f"Retrieving secret: {key}")
        
        try:
            secret = await self._fetch(key)
            logger.info(f"Successfully retrieved secret: {key}")
            return secret
        except Exception as e:
            logger.error(f"Failed to retrieve secret {key}: {e}", exc_info=True)
            raise
```

---

## Plugin Development Checklist

- [ ] Implement all abstract methods from base class
- [ ] Add comprehensive error handling
- [ ] Validate configuration with Pydantic
- [ ] Implement proper resource cleanup
- [ ] Add logging at appropriate levels
- [ ] Write unit tests (>80% coverage)
- [ ] Write integration tests
- [ ] Document configuration options
- [ ] Document error conditions
- [ ] Follow security best practices
- [ ] Add type hints throughout
- [ ] Register via entry points
- [ ] Version plugin with semver
- [ ] Create example usage
- [ ] Test with multiple Python versions

---

## Publishing Plugins

### PyPI Package Structure

```
leash-ai-my-vault/
├── src/
│   └── leash_ai_my_vault/
│       ├── __init__.py
│       ├── backends/
│       │   └── my_backend.py
│       └── py.typed
├── tests/
│   └── test_my_backend.py
├── README.md
├── LICENSE
└── pyproject.toml
```

### pyproject.toml

```toml
[project]
name = "leash-ai-myvault"
version = "0.1.0"
description = "My Vault backend for Leash AI"
dependencies = [
    "leash-ai>=0.1.0",
    "my-vault-sdk>=2.0",
]

[project.entry-points."leash_ai.secret_backends"]
my_vault = "leash_ai_my_vault.backends:MyVaultBackend"
```

### Distribution

```bash
# Build package
python -m build

# Upload to PyPI
python -m twine upload dist/*
```

### Installation

```bash
pip install leash-ai-myvault
```

Plugin automatically available:

```yaml
# config.yaml
backends:
  secrets:
    type: my_vault  # Auto-discovered
    url: https://vault.company.com
```

---

**Document Version**: 1.0  
**Last Updated**: 2025-02-07
