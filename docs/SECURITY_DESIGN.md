# Security Design Principles

## Core Principles

### 1. Least Privilege
Every component operates with the minimum permissions necessary.

**Implementation**:
- Daemon runs as non-root user
- Secrets accessed through minimal-privilege service accounts
- Tokens scoped to single resource + operation
- Package installations isolated per instance

### 2. Defense in Depth
Multiple layers of security controls.

**Layers**:
```
User Layer:      Authentication, Authorization
Transport Layer: TLS, Request Signing
Application:     Input Validation, Policy Engine
Data Layer:      Encryption at Rest, Audit Logs
Infrastructure:  File Permissions, Process Isolation
```

### 3. Fail Secure
System fails to a secure state on error.

**Examples**:
- Unknown policy? **DENY**
- Backend unavailable? **DENY**
- Invalid token? **DENY**
- Parsing error? **DENY**
- Timeout? **REVOKE**

### 4. Zero Trust
Never trust, always verify.

**Implementation**:
- Every request authenticated
- Every access authorized
- Every action logged
- No implicit trust between components

### 5. Secure by Default
Security requires no configuration.

**Defaults**:
- TLS enabled
- Authentication required
- Deny-all policy
- Audit logging enabled
- Short token TTLs (1 hour)

### 6. Privacy by Design
Minimize data collection and retention.

**Implementation**:
- Secrets never logged
- Minimal PII in audit logs
- Configurable log retention
- Data export capability

### 7. Complete Mediation
Every access checked, no caching of decisions.

**Implementation**:
- Token validated on every use
- Policy re-evaluated for each request
- No authorization caching
- Revocation immediate

### 8. Separation of Duties
No single component has complete control.

**Separation**:
- Policy creation ≠ Policy approval
- Request ≠ Approval
- Daemon ≠ Backend
- Audit log writer ≠ Audit log reader

---

## Secure Coding Standards

### Input Validation

**ALL inputs are untrusted**:
- OpenClaw requests
- Human approver responses
- Configuration files
- Environment variables
- Backend responses

**Validation Rules**:
```python
# BAD: Trust input
secret_key = request.get("key")
secret = backend.retrieve(secret_key)

# GOOD: Validate then use
secret_key = validate_secret_key(request.get("key"))
if not secret_key:
    raise ValidationError("Invalid secret key")
secret = backend.retrieve(secret_key)

def validate_secret_key(key: str) -> Optional[str]:
    """Validate and canonicalize secret key."""
    if not key or not isinstance(key, str):
        return None
    
    # Length check
    if len(key) > 200:
        return None
    
    # Path traversal check
    if ".." in key or key.startswith("/"):
        return None
    
    # Allowed characters only
    if not re.match(r"^[a-zA-Z0-9/_-]+$", key):
        return None
    
    return key
```

### Output Encoding

**Secrets must be protected in all outputs**:

```python
# BAD: Secret in log
logger.info(f"Retrieved secret: {secret_value}")

# BAD: Secret in error
raise Exception(f"Failed to use secret: {secret_value}")

# GOOD: Redact secrets
logger.info(f"Retrieved secret: {redact(secret_value)}")

# GOOD: Generic error
raise SecretAccessError("Failed to use secret")

def redact(secret: str) -> str:
    """Redact secret for logging."""
    if len(secret) <= 4:
        return "***"
    return secret[:2] + "***" + secret[-2:]
```

### Cryptography

**Use standard libraries, never roll your own**:

```python
# BAD: Custom crypto
def encrypt(data):
    return xor_cipher(data, "secret-key")

# GOOD: Use cryptography library
from cryptography.fernet import Fernet

def encrypt(data: bytes, key: bytes) -> bytes:
    """Encrypt data using Fernet (AES-128-CBC)."""
    f = Fernet(key)
    return f.encrypt(data)
```

**Random Token Generation**:
```python
# BAD: Predictable
token = hashlib.md5(str(time.time()).encode()).hexdigest()

# GOOD: Cryptographically secure
import secrets
token = secrets.token_urlsafe(32)  # 256 bits
```

### Error Handling

**Don't leak information in errors**:

```python
# BAD: Reveals existence
if not secret_exists(key):
    raise Exception(f"Secret {key} does not exist")

# GOOD: Generic error
if not secret_exists(key):
    raise SecretNotFoundError("Secret not found")

# BAD: Reveals system details
except Exception as e:
    return {"error": str(e)}

# GOOD: Generic error to client, details in logs
except Exception as e:
    logger.error(f"Internal error: {e}", exc_info=True)
    return {"error": "Internal server error"}
```

### Authentication

**Validate tokens on every request**:

```python
@app.middleware("http")
async def authenticate(request: Request, call_next):
    """Authenticate all requests."""
    # Extract token
    auth_header = request.headers.get("Authorization")
    if not auth_header or not auth_header.startswith("Bearer "):
        raise AuthenticationError("Missing or invalid token")
    
    token = auth_header[7:]  # Remove "Bearer "
    
    # Validate token
    try:
        payload = jwt.decode(
            token,
            key=JWT_SECRET,
            algorithms=["HS256"],
            options={
                "verify_signature": True,
                "verify_exp": True,
                "require": ["exp", "instance_id"],
            }
        )
    except jwt.InvalidTokenError:
        raise AuthenticationError("Invalid token")
    
    # Attach identity to request
    request.state.instance_id = payload["instance_id"]
    
    return await call_next(request)
```

### Authorization

**Check permissions before action**:

```python
async def retrieve_secret(request: SecretRequest, instance_id: str) -> str:
    """Retrieve a secret value."""
    # 1. AUTHENTICATE (already done by middleware)
    
    # 2. AUTHORIZE
    policy_result = policy_engine.evaluate(
        resource_type="secret",
        resource_id=request.key,
        requester_id=instance_id,
        rationale=request.rationale,
    )
    
    if policy_result.permission != PermissionLevel.ALLOW_AUTO:
        raise PermissionDeniedError("Access denied")
    
    # 3. AUDIT (before action)
    audit_logger.log_access_attempt(
        instance_id=instance_id,
        resource_type="secret",
        resource_id=request.key,
        status="attempting",
    )
    
    # 4. PERFORM ACTION
    try:
        secret = await secret_backend.retrieve(request.key)
    except Exception as e:
        # 5. AUDIT FAILURE
        audit_logger.log_access_attempt(
            instance_id=instance_id,
            resource_type="secret",
            resource_id=request.key,
            status="failed",
            error=str(e),
        )
        raise
    
    # 6. AUDIT SUCCESS
    audit_logger.log_access_attempt(
        instance_id=instance_id,
        resource_type="secret",
        resource_id=request.key,
        status="success",
    )
    
    return secret
```

---

## Architecture Security Patterns

### Pattern 1: Capability Tokens

**Problem**: How to grant temporary, scoped access?

**Solution**: Issue capability tokens that encode permissions.

```python
@dataclass
class CapabilityToken:
    """Token that grants specific capability."""
    token_id: str  # Unique identifier
    instance_id: str  # Who can use this
    resource_type: str  # What type of resource
    resource_id: str  # Specific resource
    operation: str  # What operation (read, write, execute)
    issued_at: datetime
    expires_at: datetime
    one_time_use: bool = True  # Can only be used once
    signature: str = ""  # HMAC signature
    
    def sign(self, secret: bytes) -> None:
        """Sign token to prevent tampering."""
        data = f"{self.token_id}:{self.instance_id}:{self.resource_type}:{self.resource_id}:{self.expires_at.isoformat()}"
        self.signature = hmac.new(
            secret,
            data.encode(),
            hashlib.sha256
        ).hexdigest()
    
    def verify(self, secret: bytes) -> bool:
        """Verify token signature."""
        data = f"{self.token_id}:{self.instance_id}:{self.resource_type}:{self.resource_id}:{self.expires_at.isoformat()}"
        expected = hmac.new(
            secret,
            data.encode(),
            hashlib.sha256
        ).hexdigest()
        
        # Constant-time comparison to prevent timing attacks
        return hmac.compare_digest(self.signature, expected)
```

### Pattern 2: Audit Middleware

**Problem**: Ensure all actions are logged.

**Solution**: Middleware that wraps all operations.

```python
class AuditMiddleware:
    """Middleware that logs all operations."""
    
    def __init__(self, audit_logger: AuditLogger):
        self.audit_logger = audit_logger
    
    async def __call__(self, operation: Callable, **kwargs) -> Any:
        """Wrap operation with audit logging."""
        operation_id = secrets.token_hex(16)
        
        # Log start
        self.audit_logger.log_operation_start(
            operation_id=operation_id,
            operation=operation.__name__,
            kwargs=self._sanitize(kwargs),
        )
        
        try:
            result = await operation(**kwargs)
            
            # Log success
            self.audit_logger.log_operation_success(
                operation_id=operation_id,
                result=self._sanitize(result),
            )
            
            return result
            
        except Exception as e:
            # Log failure
            self.audit_logger.log_operation_failure(
                operation_id=operation_id,
                error=str(e),
            )
            raise
    
    def _sanitize(self, data: Any) -> Any:
        """Remove secrets from data before logging."""
        # Implementation of secret redaction
        ...
```

### Pattern 3: Rate Limiting

**Problem**: Prevent abuse and DoS.

**Solution**: Token bucket algorithm per instance.

```python
class RateLimiter:
    """Rate limit requests per instance."""
    
    def __init__(
        self,
        rate: int = 100,  # requests per window
        window: int = 60,  # seconds
    ):
        self.rate = rate
        self.window = window
        self.buckets: Dict[str, TokenBucket] = {}
    
    async def check_limit(self, instance_id: str) -> bool:
        """Check if request is within rate limit."""
        if instance_id not in self.buckets:
            self.buckets[instance_id] = TokenBucket(
                capacity=self.rate,
                refill_rate=self.rate / self.window,
            )
        
        bucket = self.buckets[instance_id]
        
        if not bucket.consume(1):
            # Rate limit exceeded
            await self._log_rate_limit_violation(instance_id)
            return False
        
        return True
    
    async def _log_rate_limit_violation(self, instance_id: str):
        """Log rate limit violation for monitoring."""
        audit_logger.log_event(
            event_type="rate_limit_violation",
            instance_id=instance_id,
            timestamp=datetime.utcnow(),
        )
```

### Pattern 4: Secure Secrets in Memory

**Problem**: Secrets must not leak through memory dumps, swap, or logs.

**Solution**: Use secure memory handling.

```python
import ctypes
from typing import Optional

class SecureString:
    """String that is zeroed on deletion."""
    
    def __init__(self, value: str):
        # Store as byte array for easier zeroing
        self._value = bytearray(value.encode('utf-8'))
        # Lock memory to prevent swapping (Unix only)
        self._lock_memory()
    
    def _lock_memory(self):
        """Prevent memory from being swapped to disk."""
        try:
            # Unix systems
            import mlock
            mlock.mlock(self._value)
        except (ImportError, OSError):
            # Windows or insufficient permissions
            pass
    
    def get(self) -> str:
        """Get string value."""
        return self._value.decode('utf-8')
    
    def __del__(self):
        """Zero memory on deletion."""
        if hasattr(self, '_value'):
            # Overwrite with zeros
            for i in range(len(self._value)):
                self._value[i] = 0
    
    def __repr__(self) -> str:
        """Prevent accidental logging."""
        return "<SecureString [REDACTED]>"
```

---

## Security Testing Requirements

### Unit Tests

Every security control must have tests:

```python
class TestAuthenticationMiddleware:
    """Test authentication middleware."""
    
    async def test_missing_token_rejected(self):
        """Missing auth token should be rejected."""
        request = Request(headers={})
        with pytest.raises(AuthenticationError):
            await authenticate(request, lambda r: r)
    
    async def test_invalid_token_rejected(self):
        """Invalid token should be rejected."""
        request = Request(headers={"Authorization": "Bearer invalid"})
        with pytest.raises(AuthenticationError):
            await authenticate(request, lambda r: r)
    
    async def test_expired_token_rejected(self):
        """Expired token should be rejected."""
        expired_token = create_token(exp=datetime.utcnow() - timedelta(hours=1))
        request = Request(headers={"Authorization": f"Bearer {expired_token}"})
        with pytest.raises(AuthenticationError):
            await authenticate(request, lambda r: r)
```

### Integration Tests

Test security across components:

```python
async def test_unauthorized_secret_access():
    """Test that unauthorized secret access is prevented."""
    # Setup: Create instance with no permissions
    client = PermissionClient(instance_id="unauthorized-instance")
    
    # Attempt to access production secret
    with pytest.raises(PermissionDeniedError):
        await client.request_secret(
            key="production/database/password",
            rationale="Testing access",
        )
    
    # Verify audit log entry
    logs = await audit_logger.get_logs(instance_id="unauthorized-instance")
    assert len(logs) == 1
    assert logs[0]["status"] == "denied"
```

### Penetration Testing Checklist

- [ ] SQL injection (if using SQL database)
- [ ] Command injection in CLI execution
- [ ] Path traversal in secret keys
- [ ] Authentication bypass
- [ ] Authorization bypass
- [ ] Token forgery
- [ ] Rate limit bypass
- [ ] Audit log tampering
- [ ] Secrets in error messages
- [ ] Timing attacks

---

## Deployment Security

### System Requirements

```yaml
# Minimum security requirements for deployment

system:
  os:
    - name: "Ubuntu 22.04 LTS or later"
      reason: "Security patches and support"
    - name: "macOS 12.0 or later"
      reason: "Modern security features"
  
  user:
    name: "leash-ai"
    uid: 10000  # Non-privileged
    shell: "/usr/sbin/nologin"
    home: "/var/lib/leash-ai"
  
  permissions:
    daemon_binary: "0755"  # rwxr-xr-x
    config_files: "0600"   # rw-------
    policy_files: "0600"   # rw-------
    log_directory: "0700"  # rwx------
    data_directory: "0700" # rwx------

network:
  tls:
    enabled: true
    min_version: "TLS 1.3"
    ciphers:
      - "TLS_AES_256_GCM_SHA384"
      - "TLS_CHACHA20_POLY1305_SHA256"
  
  firewall:
    allow_inbound:
      - port: 8765
        source: "localhost"  # Only local connections
    deny_all_other: true
```

### Environment Hardening

```bash
#!/bin/bash
# Harden deployment environment

# 1. Create non-privileged user
sudo useradd -r -s /usr/sbin/nologin -d /var/lib/leash-ai leash-ai

# 2. Set file permissions
sudo chmod 0755 /usr/local/bin/leash
sudo chmod 0600 /etc/leash-ai/*.yaml
sudo chmod 0700 /var/lib/leash-ai
sudo chmod 0700 /var/log/leash-ai

# 3. Set ownership
sudo chown -R leash-ai:leash-ai /var/lib/leash-ai
sudo chown -R leash-ai:leash-ai /var/log/leash-ai

# 4. Disable core dumps (prevent memory leakage)
echo "* hard core 0" | sudo tee -a /etc/security/limits.conf

# 5. Enable audit logging
sudo systemctl enable auditd
sudo auditctl -w /etc/leash-ai/ -p wa -k leash-config
sudo auditctl -w /var/lib/leash-ai/ -p wa -k leash-data

# 6. Configure firewall
sudo ufw allow from 127.0.0.1 to any port 8765
sudo ufw enable
```

---

## Security Maintenance

### Regular Tasks

**Daily**:
- Monitor audit logs for anomalies
- Check rate limit violations
- Review failed authentication attempts

**Weekly**:
- Review and approve pending requests
- Analyze usage patterns
- Update threat intelligence

**Monthly**:
- Rotate secrets
- Review and update policies
- Security patch updates
- Dependency vulnerability scan

**Quarterly**:
- Penetration testing
- Security audit
- Threat model review
- Incident response drill

---

## Incident Response

### Security Incident Procedure

1. **Detect**: Anomaly detected in logs/monitoring
2. **Contain**: Revoke affected tokens, disable compromised instances
3. **Investigate**: Review audit logs, determine scope
4. **Remediate**: Patch vulnerability, rotate secrets
5. **Document**: Record incident details
6. **Review**: Update threat model and controls

### Playbooks

See `/docs/incident-response/` for detailed playbooks:
- Compromised OpenClaw Instance
- Secrets Leakage
- Unauthorized Access
- Denial of Service
- Insider Threat

---

**Document Version**: 1.0  
**Last Updated**: 2025-02-07  
**Next Review**: Before 1.0 release
