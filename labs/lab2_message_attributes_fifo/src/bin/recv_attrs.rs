use anyhow::Result;
use aws_sdk_sqs::types::MessageSystemAttributeName;
use clap::Parser;
use shared::{
    cli::{CommonArgs, merged_config, require_queue_name},
    config::build_sqs_client,
    logging, sqs,
};
use tracing::warn;

#[derive(Parser, Debug)]
#[command(name = "recv_attrs")]
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

    // Merge configs and resolve queue/url
    let default_lab_cfg = format!("{}/config.toml", env!("CARGO_MANIFEST_DIR"));
    let cfg = merged_config(&args.common, &default_lab_cfg)?;
    let client = build_sqs_client(&cfg).await?;
    let qname = require_queue_name(&args.common, &cfg)?;
    let url = sqs::get_queue_url(&client, &qname).await?;

    let wait_secs = cfg.recv_wait_secs();

    println!(
        "[recv_attrs] region={} queue={} mode={:?} wait={}s delete={}",
        cfg.runtime.region, qname, cfg.runtime.mode, wait_secs, !args.no_delete
    );
    println!("[recv_attrs] waiting for messages... (Ctrl+C to stop)");

    loop {
        let out = client
            .receive_message()
            .queue_url(&url)
            .max_number_of_messages(1)
            .wait_time_seconds(wait_secs)
            // request *all* user attributes
            .message_attribute_names("All")
            // request system attrs (e.g., SequenceNumber, MessageGroupId for FIFO)
            .message_system_attribute_names(MessageSystemAttributeName::All)
            .send()
            .await?;

        let msgs = out.messages();
        if msgs.is_empty() {
            continue;
        }

        for m in msgs {
            let mid = m.message_id().unwrap_or("unknown");
            let body = m.body().unwrap_or("");
            println!("[recv_attrs] received: message_id={} body={:?}", mid, body);

            // Print FIFO/system attributes (if present)
            if let Some(sys) = m.attributes() {
                if sys.is_empty() {
                    println!("[recv_attrs] system: (none)");
                } else {
                    for (k, v) in sys {
                        println!("[recv_attrs] system: {}={}", k, v);
                    }
                }
            }

            // Print user attributes (if any)
            if let Some(amap) = m.message_attributes() {
                if amap.is_empty() {
                    println!("[recv_attrs] attrs: (none)");
                } else {
                    for (k, v) in amap {
                        let dt = v.data_type();
                        if let Some(s) = v.string_value() {
                            println!("[recv_attrs] attrs: {}({})={:?}", k, dt, s);
                        } else {
                            // For non-string/binary types, print a generic line
                            println!("[recv_attrs] attrs: {}({})", k, dt);
                        }
                    }
                }
            }

            // Delete unless told not to
            if args.no_delete {
                warn!("--no-delete set; not deleting message_id={}", mid);
            } else if let Some(rh) = m.receipt_handle() {
                println!("[recv_attrs] deleting...");
                client
                    .delete_message()
                    .queue_url(&url)
                    .receipt_handle(rh)
                    .send()
                    .await?;
                println!("[recv_attrs] deleted message_id={}", mid);
            } else {
                warn!("missing receipt_handle; cannot delete");
            }
        }
    }
}
