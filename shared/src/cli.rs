use anyhow::{anyhow, Result};
use clap::Args as ClapArgs;

use crate::config::AppConfig;

/// Common flags shared by all Lab 1 binaries.
/// Use with `#[command(flatten)] common: CommonArgs`.
#[derive(Clone, Debug, ClapArgs)]
pub struct CommonArgs {
    /// Path to the root config (required)
    #[arg(long, default_value = "config.toml")]
    pub config: String,

    /// Path to the lab-scoped config (defaults to <lab_dir>/config.toml)
    #[arg(long)]
    pub lab_config: Option<String>,

    /// Ad-hoc override for the queue name
    #[arg(long)]
    pub queue_name: Option<String>,
}

/// Merge root + lab + env into an AppConfig.
/// `default_lab_cfg` should be "<this_lab_dir>/config.toml".
pub fn merged_config(common: &CommonArgs, default_lab_cfg: &str) -> Result<AppConfig> {
    let lab_cfg_path = common
        .lab_config
        .clone()
        .unwrap_or_else(|| default_lab_cfg.to_string());
    AppConfig::load_merged(&common.config, Some(&lab_cfg_path))
}

/// Require a queue name from either CLI or config; otherwise fail.
pub fn require_queue_name(common: &CommonArgs, cfg: &AppConfig) -> Result<String> {
    common
        .queue_name
        .clone()
        .or_else(|| cfg.sqs.queue_name.clone())
        .ok_or_else(|| {
            anyhow!(
                "Queue name is required. Pass --queue-name or set [sqs].queue_name in the lab config."
            )
        })
}
