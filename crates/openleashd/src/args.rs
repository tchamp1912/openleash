use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the configuration file
    #[arg(short, long, env = "LEASHD_CONFIG")]
    pub config: Option<std::path::PathBuf>,

    /// Port to listen on for TCP gRPC
    #[arg(short, long, env = "LEASHD_PORT")]
    pub port: Option<u16>,

    /// Host to bind for TCP gRPC
    #[arg(short, long, env = "LEASHD_HOST")]
    pub host: Option<String>,

    /// Path to the Unix Domain Socket
    #[arg(short, long, env = "LEASHD_SOCKET")]
    pub socket: Option<String>,

    /// Database URL (e.g. sqlite://leash.db)
    #[arg(short, long, env = "LEASHD_DATABASE_URL")]
    pub database_url: Option<String>,

    /// Path to the policies YAML file
    #[arg(short, long, env = "LEASHD_POLICIES_PATH")]
    pub policies: Option<String>,

    #[arg(long, env = "TELOXIDE_TOKEN")]
    pub telegram_token: Option<String>,
    #[arg(long, env = "TELEGRAM_CHAT_ID")]
    pub telegram_chat_id: Option<String>,
}
