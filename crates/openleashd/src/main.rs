use tonic::transport::Server;
use openleash_api::pb::request_service_server::RequestServiceServer;
use openleash_api::pb::task_service_server::TaskServiceServer;
use openleash_api::pb::approval_service_server::ApprovalServiceServer;
use openleash_api::pb::audit_service_server::AuditServiceServer;
use openleash_core::{policy::PolicyEngine, config::OpenLeashConfig};
use openleash_db::db::Db;
use openleash_backend_pip::PipBackend;
use openleash_backend_brew::BrewBackend;
use openleash_backend_npm::NpmBackend;
use openleash_backend_keychain::KeychainBackend;
use openleash_backend_telegram::TelegramApprovalBackend;
use openleash_backend::ApprovalBackend;
use std::sync::Arc;
use clap::Parser;

mod args;
mod daemon;
mod services;
mod worker;
mod policies;

#[cfg(test)]
mod tests;

pub use daemon::OpenLeashDaemon;
use args::Args;
use worker::{run_background_worker, run_telegram_bot};
use policies::get_default_policies;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    // 1. Load Config
    let mut config = OpenLeashConfig::load(args.config)?;

    // 2. Override with CLI flags
    if let Some(p) = args.port { config.server.tcp_port = p; }
    if let Some(h) = args.host { config.server.tcp_host = h; }
    if let Some(s) = args.socket { config.server.uds_path = s; }
    if let Some(db) = args.database_url { config.storage.database_url = db; }
    if let Some(pol) = args.policies { config.storage.policies_path = Some(pol); }

    let db = Arc::new(Db::new(&config.storage.database_url).await?);
    
    // Load Policies
    let policies = if let Some(path) = &config.storage.policies_path {
        if std::path::Path::new(path).exists() {
            let content = std::fs::read_to_string(path)?;
            serde_yaml::from_str(&content)?
        } else {
            tracing::warn!("Policy file not found at {:?}. Using defaults.", path);
            get_default_policies()
        }
    } else {
        get_default_policies()
    };

    let policy_engine = Arc::new(PolicyEngine::new(policies));
    let pip_backend = Arc::new(PipBackend);
    let brew_backend = Arc::new(BrewBackend);
    let npm_backend = Arc::new(NpmBackend::new(config.node_bootstrap_config()));
    let keychain_backend = Arc::new(KeychainBackend);

    let mut approval_backends: Vec<Arc<dyn ApprovalBackend>> = Vec::new();
    
    // Determine Telegram config (CLI flags override config file)
    let tg_token = args.telegram_token.or_else(|| config.telegram.as_ref().map(|t| t.token.clone()));
    let tg_chat_id = args.telegram_chat_id.and_then(|id| id.parse::<i64>().ok())
        .or_else(|| config.telegram.as_ref().map(|t| t.chat_id));

    if let (Some(token), Some(chat_id)) = (tg_token.clone(), tg_chat_id) {
        tracing::info!("Adding Telegram approval backend");
        approval_backends.push(Arc::new(TelegramApprovalBackend::new(token, chat_id)));
    }

    let daemon = Arc::new(OpenLeashDaemon::new(
        db,
        pip_backend,
        brew_backend,
        npm_backend,
        keychain_backend,
        policy_engine,
        approval_backends,
    ));

    if let Some(token) = tg_token {
        let bot_daemon = daemon.clone();
        tokio::spawn(async move {
            run_telegram_bot(token, bot_daemon).await;
        });
    }

    let worker_daemon = daemon.clone();
    tokio::spawn(async move {
        run_background_worker(worker_daemon).await;
    });

    // Start UDS server
    let socket_path = config.server.uds_path.clone();
    if std::path::Path::new(&socket_path).exists() {
        let _ = std::fs::remove_file(&socket_path);
    }

    let uds = tokio::net::UnixListener::bind(&socket_path)?;
    let uds_stream = tokio_stream::wrappers::UnixListenerStream::new(uds);

    let daemon_uds = daemon.clone();
    tokio::spawn(async move {
        let _ = Server::builder()
            .add_service(RequestServiceServer::from_arc(daemon_uds.clone()))
            .add_service(TaskServiceServer::from_arc(daemon_uds.clone()))
            .add_service(ApprovalServiceServer::from_arc(daemon_uds.clone()))
            .add_service(AuditServiceServer::from_arc(daemon_uds))
            .serve_with_incoming(uds_stream)
            .await;
    });

    // Start TCP server
    let addr: std::net::SocketAddr = format!("{}:{}", config.server.tcp_host, config.server.tcp_port).parse()?;
    tracing::info!(addr = %addr, socket = %socket_path, "OpenLeashDaemon listening");

    Server::builder()
        .add_service(RequestServiceServer::from_arc(daemon.clone()))
        .add_service(TaskServiceServer::from_arc(daemon.clone()))
        .add_service(ApprovalServiceServer::from_arc(daemon.clone()))
        .add_service(AuditServiceServer::from_arc(daemon))
        .serve(addr)
        .await?;

    Ok(())
}
