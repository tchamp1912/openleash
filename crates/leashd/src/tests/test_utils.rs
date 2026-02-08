use crate::{LeashDaemon, get_default_policies};
use leash_ai_core::policy::PolicyEngine;
use leash_ai_db::db::Db;
use leash_ai_backend_pip::PipBackend;
use leash_ai_backend_brew::BrewBackend;
use leash_ai_backend_npm::NpmBackend;
use leash_ai_backend_keychain::KeychainBackend;
use std::sync::Arc;

pub async fn setup_daemon() -> Arc<LeashDaemon> {
    let db = Arc::new(Db::new("sqlite::memory:").await.unwrap());
    let policies = get_default_policies();
    let policy_engine = Arc::new(PolicyEngine::new(policies));
    
    Arc::new(LeashDaemon {
        db,
        pip_backend: Arc::new(PipBackend),
        brew_backend: Arc::new(BrewBackend),
        npm_backend: Arc::new(NpmBackend::default()),
        keychain_backend: Arc::new(KeychainBackend),
        policy_engine,
        approval_backends: vec![],
    })
}
