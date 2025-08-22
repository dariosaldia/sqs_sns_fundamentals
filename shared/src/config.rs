use anyhow::{Context, Result, anyhow};
use aws_config::BehaviorVersion;
use aws_credential_types::{Credentials, provider::SharedCredentialsProvider};
use aws_sdk_sqs as sqs;
use config::{Config, Environment, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeMode {
    Local,
    Aws,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
    pub mode: RuntimeMode,
    pub region: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SqsConfig {
    pub queue_name: Option<String>,
    pub endpoint_url: Option<String>,
    pub visibility_timeout_secs: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RecvConfig {
    pub wait_secs: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub sqs: SqsConfig,
    #[serde(default)]
    pub recv: RecvConfig,
}

impl AppConfig {
    /// Load and MERGE:
    ///  - root_config (e.g., config.toml at repo root)
    ///  - lab_config  (e.g., labs/<lab>/config.toml)  — later source overrides earlier
    ///  - environment (APP_* with "__" nesting)       — highest precedence
    pub fn load_merged(root_config: &str, lab_config: Option<&str>) -> Result<Self> {
        let mut builder = Config::builder();

        // Root config (required)
        if !Path::new(root_config).exists() {
            return Err(anyhow!(
                "Root config not found at '{}'. Create it (e.g. copy config.example.toml) or pass --config <path>.",
                root_config
            ));
        }
        builder = builder.add_source(File::with_name(root_config));

        // Lab config (optional, overrides root)
        if let Some(lab_path) = lab_config {
            if Path::new(lab_path).exists() {
                builder = builder.add_source(File::with_name(lab_path));
            }
        }

        // Environment overrides (e.g., APP_RUNTIME__REGION=eu-west-1)
        builder = builder.add_source(
            Environment::with_prefix("APP")
                .separator("__")
                .try_parsing(true),
        );

        let cfg = builder.build().context("building merged config")?;
        let out: AppConfig = cfg.try_deserialize().context("deserializing AppConfig")?;
        Ok(out)
    }

    pub fn queue_name_or(&self, fallback: &str) -> String {
        self.sqs
            .queue_name
            .clone()
            .unwrap_or_else(|| fallback.to_string())
    }

    pub fn recv_wait_secs(&self) -> i32 {
        self.recv.wait_secs.unwrap_or(10)
    }
}

pub async fn build_sqs_client(cfg: &AppConfig) -> Result<sqs::Client> {
    let mut loader = aws_config::defaults(BehaviorVersion::latest())
        .region(aws_config::Region::new(cfg.runtime.region.clone()));

    // If we're on LocalStack (runtime=local) OR an explicit endpoint is provided,
    // use static dummy creds to bypass SSO/profile resolution.
    let using_localstack =
        matches!(cfg.runtime.mode, RuntimeMode::Local) || cfg.sqs.endpoint_url.is_some();

    if using_localstack {
        let creds = Credentials::new("test", "test", None, None, "localstack");
        loader = loader.credentials_provider(SharedCredentialsProvider::new(creds));
    }

    let shared_cfg = loader.load().await;

    let mut b = sqs::config::Builder::from(&shared_cfg);
    if let Some(ep) = &cfg.sqs.endpoint_url {
        b = b.endpoint_url(ep.clone());
    }
    Ok(sqs::Client::from_conf(b.build()))
}
