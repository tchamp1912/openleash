# OpenLeash - Extensibility Guide

OpenLeash is designed with a highly modular, trait-based architecture. You can easily extend the system by adding new resource backends or approval methods.

## Adding a New Resource Backend

All resources are abstracted behind traits in the `openleash-backend` crate.

### 1. Implement the `PackageBackend` Trait
If you want to support a new package manager (e.g., `apt`, `cargo`), implement this trait:

```rust
#[async_trait]
pub trait PackageBackend: Send + Sync {
    async fn install(&self, package: &str, version: Option<&str>, scope_path: &str) -> Result<()>;
    async fn uninstall(&self, scope_path: &str) -> Result<()>;
    fn executable_directory(&self, scope_path: &str) -> PathBuf;
}
```

### 2. Implement the `SecretBackend` Trait
To support a new secret store (e.g., Vault, AWS Secrets Manager):

```rust
#[async_trait]
pub trait SecretBackend: Send + Sync {
    async fn get_secret(&self, secret_id: &str) -> Result<String>;
    async fn store_secret(&self, secret_id: &str, value: &str) -> Result<()>;
}
```

## Adding a New Approval Method

Human-in-the-loop notifications are handled by the `ApprovalBackend` trait.

```rust
#[async_trait]
pub trait ApprovalBackend: Send + Sync {
    async fn notify_approval(&self, req: &ApprovalRequest) -> Result<()>;
}
```

Example implementation: `openleash-backend-telegram`. You could add `openleash-backend-slack` or `openleash-backend-email`.

## Registering Your Extension

Once you've built your crate:
1.  Add it to the `openleashd` `Cargo.toml`.
2.  Update the initialization logic in `crates/openleashd/src/main.rs` to include your new backend based on the `LeashConfig`.

## Testing Extensions

We recommend following the existing pattern:
1.  Create a `tests/` directory in your crate.
2.  Add a sanity check for initialization.
3.  Add an integration test in `openleashd` that uses your backend in a full request lifecycle.