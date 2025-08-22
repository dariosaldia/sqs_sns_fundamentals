use anyhow::Result;
use clap::Parser;
use shared::cli::{CommonArgs, merged_config, require_queue_name};
use shared::config::build_sqs_client;
use shared::{logging, sqs};
use tracing::warn;

#[derive(Parser, Debug)]
#[command(name = "recv")]
struct Args {
    #[command(flatten)]
    common: CommonArgs,

    /// Do not delete messages after receiving (observe redelivery)
    #[arg(long)]
    no_delete: bool,
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

    let wait_secs = cfg.recv_wait_secs();

    println!(
        "[recv] region={} queue={} mode={:?} wait={}s delete={}",
        cfg.runtime.region, qname, cfg.runtime.mode, wait_secs, !args.no_delete
    );
    println!("[recv] waiting for messages... (Ctrl+C to stop)");

    loop {
        let out = client
            .receive_message()
            .queue_url(&url)
            .max_number_of_messages(1)
            .wait_time_seconds(wait_secs)
            .send()
            .await?;

        let msgs = out.messages();
        if msgs.is_empty() {
            continue;
        }

        for m in msgs {
            let mid = m.message_id().unwrap_or("unknown");
            let body = m.body().unwrap_or("");
            println!("[recv] received: message_id={} body={:?}", mid, body);

            if args.no_delete {
                warn!("--no-delete set; not deleting message_id={}", mid);
                continue;
            }

            if let Some(rh) = m.receipt_handle() {
                println!("[recv] deleting...");
                client
                    .delete_message()
                    .queue_url(&url)
                    .receipt_handle(rh)
                    .send()
                    .await?;
                println!("[recv] deleted message_id={}", mid);
            } else {
                warn!("missing receipt_handle; cannot delete");
            }
        }
    }
}
