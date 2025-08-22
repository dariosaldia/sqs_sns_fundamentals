use anyhow::Result;
use clap::Parser;
use shared::cli::{CommonArgs, merged_config, require_queue_name};
use shared::config::build_sqs_client;
use shared::{logging, sqs};
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "send")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,

    /// Message body (use --msg "text") or provide as positional
    #[arg(long)]
    msg: Option<String>,

    /// Positional message fallback
    message: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::init();
    let args = Args::parse();

    let default_lab_cfg = format!("{}/config.toml", env!("CARGO_MANIFEST_DIR"));
    let cfg = merged_config(&args.common, &default_lab_cfg)?;
    let client = build_sqs_client(&cfg).await?;

    let qname = require_queue_name(&args.common, &cfg)?;
    let url = sqs::get_queue_url(&client, &qname).await?;

    let body = args
        .msg
        .or(args.message)
        .unwrap_or_else(|| "hello world".into());

    let out = client
        .send_message()
        .queue_url(&url)
        .message_body(body)
        .send()
        .await?;

    let id = out.message_id().unwrap_or("unknown");
    let md5 = out.md5_of_message_body().unwrap_or("unknown");
    info!(message_id = id, md5 = md5, "sent");
    println!("[send] sent message_id={} md5={}", id, md5);

    Ok(())
}
