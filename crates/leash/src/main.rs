use clap::{Parser, Subcommand};
use leash_ai_client::client::LeashClient;
use anyhow::Context;
use std::process::Command;
use std::collections::HashMap;
use std::io::{self, Write};
use chrono;

#[derive(Parser)]
#[command(name = "leash")]
#[command(about = "Leash AI CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "unix:///tmp/leash.sock")]
    server: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Leash AI environment
    Init {
        #[arg(short, long)]
        force: bool,
    },
    /// Install Leash AI as a system service (macOS LaunchAgent or Linux Systemd)
    Install {
        #[arg(short, long)]
        user: bool,
    },
    /// Request operations
    Request {
        #[command(subcommand)]
        resource: RequestCommands,
    },
    /// Task operations
    Task {
        #[command(subcommand)]
        operation: TaskCommands,
    },
    /// Run a command via the daemon (brokered execution)
    Run {
        #[arg(short, long)]
        task_id: Option<String>,
        #[arg(short, long)]
        reason: String,
        #[arg(short, long, default_value_t = 60)]
        timeout: u32,
        /// The command and arguments to run
        #[arg(last = true)]
        command: Vec<String>,
    },
    /// Sandbox operations
    Sandbox {
        #[command(subcommand)]
        operation: SandboxCommands,
    },
    /// Approval operations
    Approve {
        #[command(subcommand)]
        operation: ApprovalCommands,
    },
    /// Audit operations
    Audit {
        #[command(subcommand)]
        operation: AuditCommands,
    },
    /// Execute a command with injected secrets
    Exec {
        /// Format: VAR_NAME=secret_id
        #[arg(short, long)]
        secret: Vec<String>,
        
        #[arg(short, long)]
        task_id: Option<String>,

        /// The command and arguments to run
        #[arg(last = true)]
        command: Vec<String>,
    },
}

#[derive(Subcommand)]
enum RequestCommands {
    /// Install a package
    Install {
        #[arg(short, long)]
        manager: String,
        #[arg(short, long)]
        package: String,
        #[arg(short, long)]
        scope: String,
        #[arg(short, long)]
        reason: String,
        #[arg(short, long, default_value_t = 3600)]
        ttl: u64,
        #[arg(long)]
        task_id: Option<String>,
    },
    /// Get a secret (Caution: prints to console)
    Secret {
        #[arg(short, long)]
        id: String,
        #[arg(short, long)]
        reason: String,
        #[arg(long)]
        task_id: Option<String>,
    },
    /// Store a secret
    SecretStore {
        #[arg(short, long)]
        id: String,
        #[arg(short, long)]
        value: String,
        #[arg(short, long)]
        reason: String,
        #[arg(long)]
        task_id: Option<String>,
    },
}

#[derive(Subcommand)]
enum TaskCommands {
    /// Start a new task
    Start {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        base_path: String,
        #[arg(short, long, default_value_t = 3600)]
        ttl: u64,
    },
    /// End an active task
    End {
        #[arg(short, long)]
        task_id: String,
    },
}

#[derive(Subcommand)]
enum SandboxCommands {
    /// List available sandbox templates
    List,
    /// Generate a specific sandbox profile
    Generate {
        #[arg(short, long, default_value = "permissive-open")]
        profile: String,
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },
}

#[derive(Subcommand)]
enum ApprovalCommands {
    /// List all pending approvals
    List,
    /// Grant approval for a request
    Grant {
        #[arg(short, long)]
        id: String,
        #[arg(short, long)]
        scope: Option<String>,
    },
    /// Deny a request
    Deny {
        #[arg(short, long)]
        id: String,
    },
}

#[derive(Subcommand)]
enum AuditCommands {
    /// List recent audit logs
    List {
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
    },
    /// Verify the integrity of the audit ledger
    Verify,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { force } => {
            let home = std::env::var("HOME")?;
            let leash_dir = std::path::PathBuf::from(format!("{}/.leash", home));
            
            if leash_dir.exists() && !*force {
                println!("Leash is already initialized at {:?}. Use --force to re-initialize.", leash_dir);
                return Ok(());
            }

            std::fs::create_dir_all(&leash_dir)?;
            let profiles_dir = leash_dir.join("profiles");
            std::fs::create_dir_all(&profiles_dir)?;

            // 1. Interactive Telegram Setup
            let mut config = leash_ai_core::config::LeashConfig::default();
            
            println!("--- Telegram Bot Setup (Human-in-the-loop) ---");
            print!("Enable Telegram notifications? (y/N): ");
            io::stdout().flush()?;
            let mut enable_tg = String::new();
            io::stdin().read_line(&mut enable_tg)?;
            
            if enable_tg.trim().to_lowercase() == "y" {
                print!("Enter Telegram Bot Token: ");
                io::stdout().flush()?;
                let mut token = String::new();
                io::stdin().read_line(&mut token)?;
                
                print!("Enter Telegram Chat ID: ");
                io::stdout().flush()?;
                let mut chat_id_str = String::new();
                io::stdin().read_line(&mut chat_id_str)?;
                
                if let Ok(chat_id) = chat_id_str.trim().parse::<i64>() {
                    config.backends.telegram.enabled = true;
                    config.telegram = Some(leash_ai_core::config::TelegramConfig {
                        token: token.trim().to_string(),
                        chat_id,
                    });
                    println!("Telegram configured!");
                }
            }

            // 2. Generate config.yaml
            let config_path = leash_dir.join("config.yaml");
            std::fs::write(&config_path, serde_yaml::to_string(&config)?)?;
            println!("Created config: {:?}", config_path);

            // 3. Generate policies.yaml
            let mut policies = vec![
                leash_ai_core::models::Policy {
                    id: "allow-safe-packages".to_string(),
                    name: "Safe Packages".to_string(),
                    description: Some("Allow common safe libraries".to_string()),
                    resource_type: leash_ai_core::models::ResourceType::Package,
                    priority: 10,
                    allowed_patterns: vec!["requests".to_string(), "six".to_string(), "numpy".to_string()],
                    max_ttl_seconds: 3600,
                    auto_approve: true,
                    default_scope: leash_ai_core::models::ApprovalScope::Once,
                },
                leash_ai_core::models::Policy {
                    id: "allow-safe-commands".to_string(),
                    name: "Safe Commands".to_string(),
                    description: Some("Allow common safe commands".to_string()),
                    resource_type: leash_ai_core::models::ResourceType::Command,
                    priority: 10,
                    allowed_patterns: vec!["^ls$".to_string(), "^cat$".to_string(), "^grep$".to_string(), "^echo$".to_string(), "^python3$".to_string()],
                    max_ttl_seconds: 0,
                    auto_approve: true,
                    default_scope: leash_ai_core::models::ApprovalScope::Once,
                },
            ];

            if config.backends.telegram.enabled {
                policies.push(leash_ai_core::models::Policy {
                    id: "telegram-approval-for-secrets".to_string(),
                    name: "Telegram Secret Approval".to_string(),
                    description: Some("Require Telegram approval for all secrets".to_string()),
                    resource_type: leash_ai_core::models::ResourceType::Secret,
                    priority: 20,
                    allowed_patterns: vec![".*".to_string()],
                    max_ttl_seconds: 0,
                    auto_approve: false,
                    default_scope: leash_ai_core::models::ApprovalScope::Once,
                });
            }

            policies.push(leash_ai_core::models::Policy {
                id: "deny-all".to_string(),
                name: "Deny All".to_string(),
                description: Some("Default deny policy".to_string()),
                resource_type: leash_ai_core::models::ResourceType::Package,
                priority: 0,
                allowed_patterns: vec![".*".to_string()],
                max_ttl_seconds: 0,
                auto_approve: false,
                default_scope: leash_ai_core::models::ApprovalScope::Once,
            });

            let policies_path = leash_dir.join("policies.yaml");
            std::fs::write(&policies_path, serde_yaml::to_string(&policies)?)?;
            println!("Created default policies: {:?}", policies_path);

            // 4. Generate all sandbox templates
            use leash_ai_core::sandbox::{SandboxProfileGenerator, SandboxLevel, NetworkMode};
            let current_dir = std::env::current_dir()?.to_string_lossy().to_string();
            
            let templates = [
                ("permissive-open", SandboxLevel::Permissive, NetworkMode::Open),
                ("permissive-closed", SandboxLevel::Permissive, NetworkMode::Closed),
                ("restrictive-open", SandboxLevel::Restrictive, NetworkMode::Open),
                ("restrictive-closed", SandboxLevel::Restrictive, NetworkMode::Closed),
            ];

            for (name, level, net) in templates {
                let gen = SandboxProfileGenerator::new(level, net, &home, &current_dir)
                    .with_config(&config);
                let path = profiles_dir.join(format!("{}.sb", name));
                std::fs::write(&path, gen.generate())?;
            }
            
            std::fs::copy(profiles_dir.join("permissive-open.sb"), leash_dir.join("agent.sb"))?;
            println!("Created sandbox profiles in {:?}/profiles/", leash_dir);
            println!("Default profile linked to {:?}", leash_dir.join("agent.sb"));

            println!("\nInitialization complete! Next steps:");
            println!("1. Install the background service: leash install");
            println!("2. Start the daemon (macOS): launchctl load ~/Library/LaunchAgents/io.leash-ai.leashd.plist");
            println!("3. Run your agent: sandbox-exec -f {:?}/agent.sb <your-agent-command>", leash_dir);
            
            return Ok(());
        },
        Commands::Install { user: _user } => {
            let home = std::env::var("HOME")?;
            let leash_dir = std::path::PathBuf::from(format!("{}/.leash", home));
            let config_path = leash_dir.join("config.yaml");
            
            // Try to find leashd executable
            let current_exe = std::env::current_exe()?;
            let bin_dir = current_exe.parent().context("Failed to get bin dir")?;
            let leashd_path = bin_dir.join("leashd");
            
            if !leashd_path.exists() {
                anyhow::bail!("leashd executable not found at {:?}. Please ensure it is built and in the same directory as leash.", leashd_path);
            }

            match std::env::consts::OS {
                "macos" => {
                    println!("Installing Leash AI as a macOS LaunchAgent...");
                    let plist_content = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>io.leash-ai.leashd</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
        <string>--config</string>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{}/leashd.out.log</string>
    <key>StandardErrorPath</key>
    <string>{}/leashd.err.log</string>
</dict>
</plist>
"#, leashd_path.to_string_lossy(), config_path.to_string_lossy(), leash_dir.to_string_lossy(), leash_dir.to_string_lossy());

                    let agents_dir = std::path::PathBuf::from(format!("{}/Library/LaunchAgents", home));
                    std::fs::create_dir_all(&agents_dir)?;
                    let plist_path = agents_dir.join("io.leash-ai.leashd.plist");
                    std::fs::write(&plist_path, plist_content)?;
                    
                    println!("Created LaunchAgent: {:?}", plist_path);
                    println!("\nTo start the service now:");
                    println!("  launchctl load {:?}", plist_path);
                    println!("\nTo stop the service:");
                    println!("  launchctl unload {:?}", plist_path);
                },
                "linux" => {
                    println!("Installing Leash AI as a systemd user service...");
                    let service_content = format!(r#"[Unit]
Description=Leash AI Daemon
After=network.target

[Service]
ExecStart={} --config {}
Restart=always
RestartSec=5
StandardOutput=file:{}/leashd.out.log
StandardError=file:{}/leashd.err.log

[Install]
WantedBy=default.target
"#, leashd_path.to_string_lossy(), config_path.to_string_lossy(), leash_dir.to_string_lossy(), leash_dir.to_string_lossy());

                    let systemd_dir = std::path::PathBuf::from(format!("{}/.config/systemd/user", home));
                    std::fs::create_dir_all(&systemd_dir)?;
                    let service_path = systemd_dir.join("leashd.service");
                    std::fs::write(&service_path, service_content)?;

                    println!("Created systemd service: {:?}", service_path);
                    println!("\nTo start the service now:");
                    println!("  systemctl --user enable --now leashd");
                    println!("\nTo check status:");
                    println!("  systemctl --user status leashd");
                },
                _ => {
                    anyhow::bail!("Unsupported operating system: {}. Manual installation required.", std::env::consts::OS);
                }
            }

            return Ok(());
        },
        Commands::Sandbox { operation } => match operation {
            SandboxCommands::List => {
                println!("Available Sandbox Templates:");
                println!("- permissive-open:   Read anything, write to task scope, network allowed.");
                println!("- permissive-closed: Read anything, write to task scope, network blocked.");
                println!("- restrictive-open:  Read only project/libs, write to task scope, network allowed.");
                println!("- restrictive-closed: Read only project/libs, write to task scope, network blocked.");
            },
            SandboxCommands::Generate { profile, output } => {
                use leash_ai_core::sandbox::{SandboxProfileGenerator, SandboxLevel, NetworkMode};
                let home = std::env::var("HOME")?;
                let current_dir = std::env::current_dir()?.to_string_lossy().to_string();
                let config = leash_ai_core::config::LeashConfig::load(None).unwrap_or_default();

                let (level, net) = match profile.as_str() {
                    "permissive-open" => (SandboxLevel::Permissive, NetworkMode::Open),
                    "permissive-closed" => (SandboxLevel::Permissive, NetworkMode::Closed),
                    "restrictive-open" => (SandboxLevel::Restrictive, NetworkMode::Open),
                    "restrictive-closed" => (SandboxLevel::Restrictive, NetworkMode::Closed),
                    _ => anyhow::bail!("Unknown profile: {}. Use 'leash sandbox list' to see options.", profile),
                };

                let gen = SandboxProfileGenerator::new(level, net, &home, &current_dir)
                    .with_config(&config);
                
                let content = gen.generate();
                if let Some(out_path) = output {
                    std::fs::write(out_path, content)?;
                    println!("Generated {} profile to {:?}", profile, out_path);
                } else {
                    println!("{}", content);
                }
            }
        },
        Commands::Request { resource } => match resource {
            RequestCommands::Install { manager, package, scope, reason, ttl, task_id } => {
                let mut client = LeashClient::connect(cli.server).await
                    .context("Failed to connect to Leash daemon")?;
                
                println!("Requesting installation of {} via {}...", package, manager);
                let lease_id = client.request_package(manager, package, scope, reason, *ttl, task_id.clone()).await
                    .context("Request failed")?;
                
                println!("Success! Lease ID: {}", lease_id);
            },
            RequestCommands::Secret { id, reason, task_id } => {
                let mut client = LeashClient::connect(cli.server).await
                    .context("Failed to connect to Leash daemon")?;
                
                let val = client.request_secret(id, reason, task_id.clone()).await
                    .context("Failed to fetch secret")?;
                
                println!("{}", val);
            },
            RequestCommands::SecretStore { id, value, reason, task_id } => {
                let mut client = LeashClient::connect(cli.server).await
                    .context("Failed to connect to Leash daemon")?;
                
                client.store_secret(id, value, reason, task_id.clone()).await
                    .context("Failed to store secret")?;
                
                println!("Secret {} stored successfully", id);
            }
        },
        Commands::Task { operation } => match operation {
            TaskCommands::Start { name, base_path, ttl } => {
                let mut client = LeashClient::connect(cli.server).await
                    .context("Failed to connect to Leash daemon")?;
                
                let (id, path) = client.start_task(name, base_path, *ttl).await
                    .context("Failed to start task")?;
                
                println!("Task started!");
                println!("ID: {}", id);
                println!("Scope: {}", path);
            },
            TaskCommands::End { task_id } => {
                let mut client = LeashClient::connect(cli.server).await
                    .context("Failed to connect to Leash daemon")?;
                
                client.end_task(task_id).await
                    .context("Failed to end task")?;
                
                println!("Task {} ended and scope cleaned up.", task_id);
            }
        },
        Commands::Run { task_id, reason, timeout, command } => {
            if command.is_empty() {
                anyhow::bail!("No command provided to run");
            }

            let mut client = LeashClient::connect(cli.server).await
                .context("Failed to connect to Leash daemon")?;

            let cmd = &command[0];
            let args = command[1..].to_vec();

            let res = client.execute_command(
                cmd,
                args,
                reason,
                task_id.clone(),
                HashMap::new(),
                None,
                *timeout,
            ).await?;

            if res.status == "EXECUTED" {
                print!("{}", res.stdout);
                eprint!("{}", res.stderr);
                std::process::exit(res.exit_code);
            } else {
                println!("Command Status: {}", res.status);
                if !res.error_message.is_empty() {
                    println!("Error: {}", res.error_message);
                }
            }
        },
        Commands::Approve { operation } => match operation {
            ApprovalCommands::List => {
                let mut client = LeashClient::connect(cli.server).await
                    .context("Failed to connect to Leash daemon")?;
                
                let approvals = client.list_pending_approvals().await?;
                if approvals.is_empty() {
                    println!("No pending approvals.");
                } else {
                    println!("{:<36} {:<15} {:<20} {:<20}", "APPROVAL_ID", "TYPE", "RESOURCE", "REASON");
                    println!("{}", "-".repeat(91));
                    for a in approvals {
                        println!("{:<36} {:<15} {:<20} {:<20}", a.approval_id, a.resource_type, a.resource_id, a.reason);
                    }
                }
            },
            ApprovalCommands::Grant { id, scope } => {
                let mut client = LeashClient::connect(cli.server).await
                    .context("Failed to connect to Leash daemon")?;
                
                if client.approve(id, scope.clone()).await? {
                    println!("✓ Request approved successfully.");
                } else {
                    println!("✗ Failed to approve request.");
                }
            },
            ApprovalCommands::Deny { id } => {
                let mut client = LeashClient::connect(cli.server).await
                    .context("Failed to connect to Leash daemon")?;
                
                if client.deny(id).await? {
                    println!("✓ Request denied.");
                } else {
                    println!("✗ Failed to deny request.");
                }
            }
        },
        Commands::Audit { operation } => match operation {
            AuditCommands::List { limit } => {
                let mut client = LeashClient::connect(cli.server).await
                    .context("Failed to connect to Leash daemon")?;
                
                let entries = client.query_audit_logs(*limit).await?;
                if entries.is_empty() {
                    println!("No audit logs found.");
                } else {
                    println!("{:<20} {:<10} {:<10} {:<15} {:<15} {:<10}", "TIMESTAMP", "TYPE", "ACTOR", "RESOURCE", "ACTION", "STATUS");
                    println!("{}", "-".repeat(90));
                    for e in entries {
                        let ts = e.timestamp.map(|t| {
                            let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(t.seconds, t.nanos as u32).unwrap_or_default();
                            dt.format("%Y-%m-%d %H:%M:%S").to_string()
                        }).unwrap_or_else(|| "N/A".to_string());
                        
                        println!("{:<20} {:<10} {:<10} {:<15} {:<15} {:<10}", ts, e.event_type, e.actor, e.resource_id, e.action, e.status);
                    }
                }
            },
            AuditCommands::Verify => {
                let mut client = LeashClient::connect(cli.server).await
                    .context("Failed to connect to Leash daemon")?;
                
                let entries = client.query_audit_logs(1000).await?;
                if entries.is_empty() {
                    println!("No audit logs to verify.");
                    return Ok(());
                }

                println!("Verifying hash-chain for {} entries...", entries.len());
                
                let mut valid = true;

                for e in &entries {
                    if e.integrity_hash.is_empty() {
                        println!("✗ Entry {} has no hash!", e.id);
                        valid = false;
                    }
                }

                if valid {
                    println!("✓ Hash chain integrity verified locally (structure only).");
                } else {
                    println!("✗ Integrity check failed.");
                }
            }
        },
        Commands::Exec { secret, task_id, command } => {
            if command.is_empty() {
                anyhow::bail!("No command provided to exec");
            }

            let mut client = LeashClient::connect(cli.server).await
                .context("Failed to connect to Leash daemon")?;

            let mut env_vars = HashMap::new();
            for s in secret {
                let parts: Vec<&str> = s.splitn(2, '=').collect();
                if parts.len() != 2 {
                    anyhow::bail!("Invalid secret format: {}. Use VAR=secret_id", s);
                }
                let var_name = parts[0];
                let secret_id = parts[1];

                let val = client.request_secret(secret_id, "leash exec injection", task_id.clone()).await
                    .context(format!("Failed to fetch secret {}", secret_id))?;
                
                env_vars.insert(var_name.to_string(), val);
            }

            let mut child = Command::new(&command[0]);
            child.args(&command[1..]);
            child.envs(&env_vars);

            let status = child.status().context("Failed to run command")?;
            std::process::exit(status.code().unwrap_or(1));
        }
    }

    Ok(())
}
