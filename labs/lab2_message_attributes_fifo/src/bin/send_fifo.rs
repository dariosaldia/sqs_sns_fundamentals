use anyhow::{anyhow, Result};
use clap::Parser;
use shared::{
    cli::{merged_config, require_queue_name, CommonArgs},
    config::build_sqs_client,
    logging, sqs,
};

#[derive(Parser, Debug)]
#[command(name = "send_fifo")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,

    /// Message body (use --msg "text") or provide as positional
    #[arg(long)]
    msg: Option<String>,

    /// Positional message (fallback)
    message: Option<String>,

    /// FIFO MessageGroupId (required for FIFO queues)
    #[arg(long, value_name = "GROUP")]
    group: String,

    /// Optional MessageDeduplicationId (if you want explicit dedup)
    #[arg(long, value_name = "DEDUP_ID")]
    dedup: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::init();
    let args = Args::parse();

    // Merge configs and resolve queue/url
    let default_lab_cfg = format!("{}/config.toml", env!("CARGO_MANIFEST_DIR"));
    let cfg = merged_config(&args.common, &default_lab_cfg)?;
    let client = build_sqs_client(&cfg).await?;
    let qname = require_queue_name(&args.common, &cfg)?;

    // Guard: FIFO names must end with .fifo
    if !qname.ends_with(".fifo") {
        return Err(anyhow!(
            "send_fifo requires a FIFO queue (name must end with .fifo). Current: {}",
            qname
        ));
    }

    let url = sqs::get_queue_url(&client, &qname).await?;
    let body = args.msg.or(args.message).unwrap_or_else(|| "hello".into());

    let mut req = client
        .send_message()
        .queue_url(&url)
        .message_body(body)
        .message_group_id(args.group);

    if let Some(d) = args.dedup {
        req = req.message_deduplication_id(d);
    }

    let out = req.send().await?;
    let id = out.message_id().unwrap_or("unknown");
    // SequenceNumber is present for FIFO; don’t fail if missing.
    let seq = out.sequence_number().unwrap_or("-");
    println!("[send_fifo] sent message_id={} sequence={}", id, seq);
    Ok(())
}
