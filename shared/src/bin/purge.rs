use anyhow::{Context, Result};
use clap::Parser;
use shared::cli::{CommonArgs, merged_config, require_queue_name};
use shared::config::build_sqs_client;
use shared::{logging, sqs};
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "purge")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::init();
    let args = Args::parse();

    let default_lab_cfg = format!("{}/config.toml", env!("CARGO_MANIFEST_DIR"));
    let cfg = merged_config(&args.common, &default_lab_cfg)?;
    let client = build_sqs_client(&cfg).await?;

    let qname = require_queue_name(&args.common, &cfg)?;
    let url = sqs::get_queue_url(&client, &qname)
        .await
        .with_context(|| format!("queue {} not found", qname))?;

    sqs::purge_queue(&client, &url).await?;
    info!("Purged queue: {}", url);
    Ok(())
}
