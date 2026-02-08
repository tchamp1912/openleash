use crate::{OpenLeashDaemon, get_default_policies};
use openleash_core::policy::PolicyEngine;
use openleash_db::db::Db;
use openleash_backend_pip::PipBackend;
use openleash_backend_brew::BrewBackend;
use openleash_backend_npm::NpmBackend;
use openleash_backend_keychain::KeychainBackend;
use std::sync::Arc;

pub async fn setup_daemon() -> Arc<OpenLeashDaemon> {
    let db = Arc::new(Db::new("sqlite::memory:").await.unwrap());
    let policies = get_default_policies();
    let policy_engine = Arc::new(PolicyEngine::new(policies));
    
    Arc::new(OpenLeashDaemon {
        db,
        pip_backend: Arc::new(PipBackend),
        brew_backend: Arc::new(BrewBackend),
        npm_backend: Arc::new(NpmBackend::default()),
        keychain_backend: Arc::new(KeychainBackend),
        policy_engine,
        approval_backends: vec![],
    })
}
