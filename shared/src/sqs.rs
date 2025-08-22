use anyhow::{Context, Result, anyhow};
use aws_sdk_sqs::Client;
use aws_sdk_sqs::types::QueueAttributeName;

use crate::config::SqsConfig;

pub async fn get_queue_url(client: &Client, queue_name: &str) -> Result<String> {
    let out = client
        .get_queue_url()
        .queue_name(queue_name)
        .send()
        .await
        .with_context(|| format!("getting queue url for {queue_name}"))?;

    out.queue_url()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("queue url missing in response"))
}

pub async fn create_queue(client: &Client, sqs_cfg: &SqsConfig) -> Result<String> {
    let name = sqs_cfg
        .queue_name
        .as_deref()
        .ok_or_else(|| anyhow!("SQS queue_name is required in [sqs].queue_name or --queue-name"))?;

    let mut req = client.create_queue().queue_name(name);

    if let Some(vt) = sqs_cfg.visibility_timeout_secs {
        req = req.attributes(QueueAttributeName::VisibilityTimeout, vt.to_string());
    }

    let out = req
        .send()
        .await
        .with_context(|| format!("creating queue {name}"))?;

    out.queue_url()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("queue url missing after create"))
}

pub async fn purge_queue(client: &Client, queue_url: &str) -> Result<()> {
    client
        .purge_queue()
        .queue_url(queue_url)
        .send()
        .await
        .context("purging queue")?;
    Ok(())
}

pub async fn delete_queue(client: &Client, queue_url: &str) -> Result<()> {
    client
        .delete_queue()
        .queue_url(queue_url)
        .send()
        .await
        .context("deleting queue")?;
    Ok(())
}

/// For debugging: fetch and print approximate metrics
pub async fn print_attrs(client: &Client, queue_url: &str) -> Result<()> {
    let out = client
        .get_queue_attributes()
        .queue_url(queue_url)
        .attribute_names(QueueAttributeName::All)
        .send()
        .await
        .context("get_queue_attributes")?;

    if let Some(map) = out.attributes() {
        for (k, v) in map {
            println!("[attr] {k} = {v}");
        }
    }
    Ok(())
}
