use anyhow::{Result, anyhow};
use aws_sdk_sqs::types::MessageAttributeValue;
use clap::Parser;
use shared::{
    cli::{CommonArgs, merged_config, require_queue_name},
    config::build_sqs_client,
    logging, sqs,
};

#[derive(Parser, Debug)]
#[command(name = "send_attrs")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,

    /// Message body (use --msg "text") or provide as positional
    #[arg(long)]
    msg: Option<String>,

    /// Positional message (fallback)
    message: Option<String>,

    /// Add attribute as key=value (repeatable)
    /// Example: --attr event_type=user.created --attr tenant=acme
    #[arg(long = "attr")]
    attrs: Vec<String>,

    /// For FIFO queues: MessageGroupId
    #[arg(long, value_name = "GROUP")]
    group: Option<String>,

    /// For FIFO queues: MessageDeduplicationId (optional)
    #[arg(long, value_name = "DEDUP_ID")]
    dedup: Option<String>,
}

fn parse_attr(kv: &str) -> Result<(String, String)> {
    let (k, v) = kv
        .split_once('=')
        .ok_or_else(|| anyhow!("Invalid --attr '{}'. Use key=value.", kv))?;
    if k.is_empty() {
        return Err(anyhow!("Attribute key cannot be empty"));
    }
    Ok((k.to_string(), v.to_string()))
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
    let url = sqs::get_queue_url(&client, &qname).await?;

    let body = args.msg.or(args.message).unwrap_or_else(|| "hello".into());

    // Start request
    let mut req = client.send_message().queue_url(&url).message_body(body);

    // Add attributes (String type)
    for kv in &args.attrs {
        let (k, v) = parse_attr(kv)?;
        let mval = MessageAttributeValue::builder()
            .data_type("String")
            .string_value(v)
            .build()?;
        req = req.message_attributes(k, mval);
    }

    // Determine if this is a FIFO queue
    let is_fifo = qname.ends_with(".fifo") || cfg.sqs.fifo.unwrap_or(false);

    // FIFO-only fields
    if is_fifo {
        let group = args
            .group
            .clone()
            .ok_or_else(|| anyhow!("This queue is FIFO; --group <MessageGroupId> is required."))?;
        req = req.message_group_id(group);
        if let Some(d) = args.dedup {
            req = req.message_deduplication_id(d);
        }
    } else {
        // On Standard queues, ignore FIFO flags if provided
        if args.group.is_some() || args.dedup.is_some() {
            eprintln!(
                "[send-attrs] Warning: --group/--dedup ignored because {} is a Standard queue",
                qname
            );
        }
    }

    let out = req.send().await?;
    let id = out.message_id().unwrap_or("unknown");
    println!("[send_attrs] sent message_id={}", id);
    Ok(())
}
