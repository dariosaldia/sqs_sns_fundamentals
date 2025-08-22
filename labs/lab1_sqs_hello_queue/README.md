
# Lab 1 – SQS “Hello Queue”: Send / Receive / Delete (Standard)

Minimal SQS flow using the AWS SDK for Rust: send a message to a **Standard** queue, receive it, and **acknowledge** it by deleting it.

This lab clarifies how SQS differs from Kafka:
- No partitions or offsets.
- Processing is confirmed by **`DeleteMessage`** (receive alone is not an ack).
- **At-least-once** delivery (duplicates possible, order is best-effort on Standard queues).

## Purpose
Establish the baseline for all subsequent labs: create a queue, send a message, consume it, and explicitly delete it. Also observe re‑delivery when a message isn’t deleted before its visibility timeout.

## What you build / use
- **Shared executables** (in the `shared` crate): `bootstrap`, `recv`, `send`, `purge`, `teardown`.
- **Config files:**
  - Root config (required): `config.toml` at repo root.
  - Lab config (optional, merged over root): `labs/lab1_sqs_hello_queue/config.toml`.
- **LocalStack** (optional) to run locally; or real AWS if you prefer.

## Prerequisites
- Rust (stable) + Cargo
- `make`
- Docker (only if using LocalStack)
- A root **`config.toml`** (required). Example for LocalStack:
- A **lab‑scoped** config for Lab 1 (merged on top of root):

> The tooling fails fast if the root config is missing. The queue name must be provided either via the lab config (`[sqs].queue_name`) or with `--queue-name` on the CLI.

## Start LocalStack (optional)
```bash
make up
```

## Commands (run from repo root)
**Bootstrap (create/verify the queue)**
```bash
make LAB=lab1_sqs_hello_queue bootstrap
```

**Start the consumer (Terminal A)**
```bash
make LAB=lab1_sqs_hello_queue recv
```
No delete (to observe re‑delivery after visibility timeout):
```bash
make LAB=lab1_sqs_hello_queue recv ARGS="--no-delete"
```

**Send a message (Terminal B)**
```bash
make LAB=lab1_sqs_hello_queue send MSG="hello world"
```

**Purge the queue (remove all messages)**
```bash
make LAB=lab1_sqs_hello_queue purge
```

**Teardown (delete the queue)**
```bash
make LAB=lab1_sqs_hello_queue teardown
```

**Stop LocalStack (if used)**
```bash
make down
```

## Expected terminal sessions

**Terminal A — Consumer**
```
$ make LAB=lab1_sqs_hello_queue recv
[recv] region=eu-central-1 queue=lab1-hello-queue mode=Local wait=10s delete=true
[recv] waiting for messages... (Ctrl+C to stop)
[recv] received: message_id=9f3b... body="hello world"
[recv] deleting...
[recv] deleted message_id=9f3b...
```

**Terminal B — Producer**
```
$ make LAB=lab1_sqs_hello_queue send MSG="hello world"
[send] sent message_id=9f3b... md5=5eb63bbbe01e...
```

**Optional: at‑least‑once observation**
Run the consumer with `--no-delete`, send a message, wait for the **visibility timeout** (~30s by default) and see the message delivered again.

## Key takeaways
- **Ack = Delete**: `DeleteMessage` marks processing complete; receiving a message does not.
- **At‑least‑once**: Duplicates can happen; consumers should be **idempotent**.
- **No offsets/partitions**: SQS uses visibility timeout and explicit deletes, not offsets.
- **Ordering (Standard)**: Best‑effort ordering (strict ordering requires FIFO queues; covered later).
- **Visibility timeout**: If a message isn’t deleted in time, it becomes visible again and may be re‑delivered.

## Common misunderstandings
- “Receiving a message acknowledges it.” -> **False**. Only deleting acknowledges.
- “Duplicates indicate a bug.” -> **Not necessarily**. At‑least‑once can deliver duplicates.
- “SQS guarantees global ordering.” -> **False** for Standard queues.

## Further reading
- AWS SQS: SendMessage / ReceiveMessage / DeleteMessage basics
- Visibility timeout and re‑delivery patterns
- Standard vs FIFO queue behavior and limits

## Cleanup
```bash
make LAB=lab1_sqs_hello_queue purge
make LAB=lab1_sqs_hello_queue teardown
make down   # if using LocalStack
```

## Repo layout (relevant parts)
```
Makefile
config.toml                      # root (required)
/shared                          # shared crate with reusable bins
  src/bin/{bootstrap,recv,send,purge,teardown}.rs
/labs
  /lab1_sqs_hello_queue
    README.md                    # this file
    config.toml                  # lab‑specific overrides
```

