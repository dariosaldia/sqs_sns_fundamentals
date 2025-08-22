use anyhow::Result;
use clap::Parser;
use shared::cli::{CommonArgs, merged_config, require_queue_name};
use shared::config::build_sqs_client;
use shared::{logging, sqs};
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(name = "bootstrap")]
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
    let sqs_client = build_sqs_client(&cfg).await?;

    let qname = require_queue_name(&args.common, &cfg)?;

    let url = match sqs::get_queue_url(&sqs_client, &qname).await {
        Ok(u) => {
            info!("Queue already exists: {u}");
            u
        }
        Err(_) => {
            warn!("Queue not found, creating: {}", qname);
            let u = sqs::create_queue(&sqs_client, &cfg.sqs).await?;
            info!("Created queue: {u}");
            u
        }
    };

    sqs::print_attrs(&sqs_client, &url).await.ok();
    Ok(())
}
