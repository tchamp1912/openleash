# Code Hygiene Report

## Dead Code

### Unused Types in `openleash-core/src/models.rs`

1. **`RequestState` enum** (lines 29-40)
   - Status: Not used anywhere in the codebase
   - Recommendation: Mark with `#[allow(dead_code)]` if planned for future use, or remove if not needed
   - Note: May be intended for future request state tracking

2. **`CapabilityGrant` struct** (lines 134-148)
   - Status: Not used anywhere in the codebase
   - Recommendation: Mark with `#[allow(dead_code)]` - mentioned in ARCHITECTURE.md as planned feature
   - Note: JWT-style capability tokens are documented but not yet implemented

### Unused Dependencies

1. **`config = "0.14"`** in workspace `Cargo.toml` (line 53)
   - Status: Listed but never imported/used
   - Recommendation: **Remove** - we use `serde_yaml` directly for config loading
   - Also listed in `openleashd/Cargo.toml` but not used there either

## Code Quality Issues

### Missing Re-exports in `openleash-core`

- **Issue**: Only `LeashError` and `Result` are re-exported from `lib.rs`
- **Impact**: Callers must use `openleash_core::models::...`, `openleash_core::config::...`, etc.
- **Recommendation**: Add `pub use models::*;` or selective re-exports for commonly used types
- **Priority**: Low (cosmetic, but improves ergonomics)

### Unused Imports (Fixed)

- [FIXED] `ApprovalScope` in `daemon.rs` - removed
- [FIXED] `LeaseStatus` in `worker.rs` - removed
- [FIXED] Missing `SecretBackend` trait imports - fixed

## Recommendations Summary

### High Priority
1. **Remove unused `config` dependency** from workspace and `openleashd/Cargo.toml`

### Medium Priority
2. **Mark `RequestState` and `CapabilityGrant`** with `#[allow(dead_code)]` and add TODO comments if they're planned

### Low Priority
3. **Add re-exports** in `openleash-core/lib.rs` for better ergonomics
4. **Consider adding `#[deny(dead_code)]`** to catch future dead code (after fixing current issues)
