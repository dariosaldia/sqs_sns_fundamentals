# Lab 2 – Message Attributes & FIFO Ordering

Attach **message attributes** and demonstrate **per-group ordering** on a **FIFO** queue. Attributes are user-defined metadata separate from the message body. FIFO ordering uses `MessageGroupId` and either a `MessageDeduplicationId` or content-based deduplication.

## Purpose

- Send and receive messages with **attributes**.
- Use a **FIFO** queue and `MessageGroupId` to guarantee **ordering within a group**.
- Observe that Standard queues provide **best-effort** ordering only.

## What is used

- **Shared executables** (from `shared`): `bootstrap`, `purge`, `teardown`.
- **Lab executables** (in this lab):
  - `send_attrs` — send a message with one or more attributes.
  - `send-fifo` — send to a FIFO queue with `--group` (and optional `--dedup`).
  - `recv_attrs` — receive and print body + user attributes and system attributes (e.g., `MessageGroupId`, `SequenceNumber`).

## Prerequisites

- Lab 1 environment working (root config, Makefile targets).
- Root **`config.toml`** (required), example for LocalStack:

```toml
[runtime]
mode   = "local"
region = "eu-central-1"

[endpoints]                 # omit for real AWS
sqs = "http://localhost:4566"

[recv]
wait_secs = 10
```

- Lab 2 config at `labs/lab2_message_attributes_fifo/config.toml` (FIFO queue name must end with `.fifo`). Choose one deduplication approach:

**Option A — content-based deduplication (no per-message dedup id needed):**
```toml
[sqs]
queue_name = "lab2-fifo-queue.fifo"
fifo = true
content_based_dedup = true
```

**Option B — explicit dedup id per message:**
```toml
[sqs]
queue_name = "lab2-fifo-queue.fifo"
fifo = true
# content_based_dedup omitted; pass --dedup on sends
```

> A FIFO queue requires the `.fifo` suffix and a `MessageGroupId` on every send.

## Commands (from repo root)

### About Deduplication

FIFO queues require a deduplication strategy:

- **MessageDeduplicationId** (`--dedup` flag in the lab commands):  
  If you reuse the same value within ~5 minutes for the same `MessageGroupId`, SQS suppresses duplicates. Use unique IDs (e.g., UUIDs, timestamps) if you want every message delivered.

- **Content-based deduplication** (`content_based_dedup = true` in the lab config):  
  SQS automatically hashes the **message body only**. If two messages in the same group have the same body within ~5 minutes, later ones are dropped — even if attributes differ.

This mechanism ensures FIFO queues achieve *exactly-once processing semantics* per group.

### 1) Bootstrap resources
```bash
make LAB=lab2_message_attributes_fifo bootstrap
```

### 2) Start the consumer that prints attributes (Terminal A)
```bash
make LAB=lab2_message_attributes_fifo run BIN=recv_attrs
```

### 3) Send a message with attributes (Terminal B)

- With **content-based dedup** enabled:
```bash
make LAB=lab2_message_attributes_fifo run BIN=send_attrs -- \
  ARGS='--group A --msg "user created" --attr event_type=user.created --attr tenant=acme'
```

- Without content-based dedup (pass a dedup id):
```bash
make LAB=lab2_message_attributes_fifo run BIN=send_attrs -- \
  ARGS='--group A --dedup user-created-1 --msg "user created" --attr event_type=user.created --attr tenant=acme'
```

### 4) Demonstrate ordering across two groups
```bash
make LAB=lab2_message_attributes_fifo run BIN=send_fifo -- ARGS='--group A --msg "A1" --dedup a1'
make LAB=lab2_message_attributes_fifo run BIN=send_fifo -- ARGS='--group A --msg "A2" --dedup a2'
make LAB=lab2_message_attributes_fifo run BIN=send_fifo -- ARGS='--group B --msg "B1" --dedup b1'
make LAB=lab2_message_attributes_fifo run BIN=send_fifo -- ARGS='--group B --msg "B2" --dedup b2'
make LAB=lab2_message_attributes_fifo run BIN=send_fifo -- ARGS='--group A --msg "A3" --dedup a3'
```

> If content-based dedup is enabled, `--dedup` can be omitted.

## Expected output

**Terminal A — `recv_attrs`**
```
[recv-attrs] region=eu-central-1 queue=lab2-fifo-queue.fifo mode=Local wait=10s delete=true
[recv-attrs] waiting for messages... (Ctrl+C to stop)
[recv-attrs] received: message_id=... body="user created"
[recv-attrs] system: MessageGroupId=A
[recv-attrs] system: SequenceNumber=00000000000000000001
[recv-attrs] attrs: event_type(String)="user.created"
[recv-attrs] attrs: tenant(String)="acme"
```

**Terminal B — `send_attrs` / `send_fifo`**
```
[send-attrs] sent message_id=...
[send-fifo]  sent message_id=... sequence=00000000000000000002
```

**Observation**
- Messages for `group=A` are delivered in send order (A1 → A2 → A3) even if interleaved with `group=B` messages.
- Attributes are listed separately from the body; they must be requested explicitly by the consumer.
- Standard queues do not guarantee ordering; FIFO queues do (per group).

## Key takeaways

- **Attributes are metadata** separate from the body; retrieve them explicitly.
- **FIFO ordering is per `MessageGroupId`**; different groups may interleave.
- **Deduplication is required on FIFO**: enable content-based dedup or pass `MessageDeduplicationId` (`--dedup`).
- **Standard queues**: at-least-once delivery and best-effort ordering.

## Common misunderstandings

- “Attributes are part of the body.” -> Attributes are stored separately and fetched via `message_attribute_names`.
- “FIFO guarantees global order.” -> Order is guaranteed **within a group**, not across all messages.
- “FIFO does not need a group id.” -> `MessageGroupId` is required on every FIFO send.

## Cleanup

```bash
make LAB=lab2_message_attributes_fifo purge
make LAB=lab2_message_attributes_fifo teardown
```
