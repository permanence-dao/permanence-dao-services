use serde::Deserialize;
use std::fmt;

const DEFAULT_CONFIG_DIR: &str = "./config";
const DEV_CONFIG_DIR: &str = "../_config";

#[derive(Clone, Debug, Deserialize)]
pub enum Environment {
    Development,
    Test,
    Production,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Environment::Development => write!(f, "Development"),
            Environment::Test => write!(f, "Test"),
            Environment::Production => write!(f, "Production"),
        }
    }
}

impl From<&str> for Environment {
    fn from(env: &str) -> Self {
        match env.to_lowercase().as_str() {
            "testing" | "test" => Environment::Test,
            "production" | "prod" => Environment::Production,
            "development" | "dev" => Environment::Development,
            _ => panic!("Unknown environment: {env}"),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct CommonConfig {
    pub recovery_retry_seconds: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HTTPConfig {
    pub request_timeout_seconds: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SubstrateConfig {
    pub connection_timeout_seconds: u64,
    pub request_timeout_seconds: u64,
    pub gov_proxy_seed_phrase: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReferendumImporterConfig {
    pub opensquare_space: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TelegramConfig {
    pub api_id: String,
    pub api_hash: String,
    pub api_token: String,
    pub chat_id: i64,
    pub bot_username: String,
    pub bot_chat_thread_id: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OpenAPIConfig {
    pub organization: String,
    pub project: String,
    pub api_key: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct VoterConfig {
    pub voting_admin_usernames: String,
    pub polkadot_real_account_address: String,
    pub polkadot_dv_delegation_account_address: String,
    pub polkadot_proxy_account_seed_phrase: String,
    pub kusama_real_account_address: String,
    pub kusama_dv_delegation_account_address: String,
    pub kusama_proxy_account_seed_phrase: String,
    pub sleep_seconds: u64,
    pub min_referendum_id: u32,
    pub member_count: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LogConfig {
    pub pdao_level: String,
    pub other_level: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PostgreSQLConfig {
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub username: String,
    pub password: String,
    pub pool_max_connections: u32,
    pub connection_timeout_seconds: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MetricsConfig {
    pub host: String,
    pub referendum_importer_port: u16,
    pub voter_port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ArchiveConfig {
    pub working_dir_path: String,
    pub python_bin_path: String,
    pub script_path: String,
    pub temp_file_dir_path: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub common: CommonConfig,
    pub http: HTTPConfig,
    pub log: LogConfig,
    pub postgres: PostgreSQLConfig,
    pub substrate: SubstrateConfig,
    pub metrics: MetricsConfig,
    pub referendum_importer: ReferendumImporterConfig,
    pub telegram: TelegramConfig,
    pub openai: OpenAPIConfig,
    pub voter: VoterConfig,
    pub archive: ArchiveConfig,
}

impl Config {
    fn new() -> Result<Self, config::ConfigError> {
        let env = Environment::from(
            std::env::var("PDAO_ENV")
                .unwrap_or_else(|_| "Production".into())
                .as_str(),
        );
        let config_dir = if cfg!(debug_assertions) {
            std::env::var("PDAO_CONFIG_DIR").unwrap_or_else(|_| DEV_CONFIG_DIR.into())
        } else {
            std::env::var("PDAO_CONFIG_DIR").unwrap_or_else(|_| DEFAULT_CONFIG_DIR.into())
        };
        let config = config::Config::builder()
            .set_default("env", env.to_string())?
            .add_source(config::File::with_name(&format!("{config_dir}/base")))
            .add_source(config::File::with_name(&format!(
                "{}/env/{}",
                config_dir,
                env.to_string().to_lowercase()
            )))
            .add_source(config::Environment::with_prefix("pdao").separator("__"))
            .build()?;
        config.try_deserialize()
    }

    pub fn get_postgres_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode=disable",
            self.postgres.username,
            self.postgres.password,
            self.postgres.host,
            self.postgres.port,
            self.postgres.database_name,
        )
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new().expect("Config can't be loaded.")
    }
}
