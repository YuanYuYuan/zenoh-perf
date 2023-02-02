mod opts;

use anyhow::{anyhow, Result};
use clap::Parser;
use itertools::chain;
use log::{info, trace};
use opts::Opts;
use rumqttc_test::{create_client, DEFAULT_QOS};
use std::{iter, process};
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let opts = Opts::parse();
    let timeout_dur = opts.timeout;

    let future = run_producer(opts);
    if let Some(timeout_dur) = timeout_dur {
        timeout(timeout_dur, future)
            .await
            .map_err(|_| anyhow!("timeout"))??;
    } else {
        future.await?;
    }

    Ok(())
}

async fn run_producer(opts: Opts) -> Result<()> {
    let producer_id = process::id();
    info!("Start producer {} on topic {}", producer_id, opts.topic);

    // Configure the client
    let (client, mut event_loop) =
        create_client(env!("CARGO_BIN_NAME"), opts.broker, opts.payload_size)?;

    // Spin the event loop to drive publishing work
    let spin_worker = async move {
        loop {
            event_loop.poll().await?;
        }

        #[allow(unreachable_code)]
        anyhow::Ok(())
    };

    // Publishing loop
    let publish_worker = async move {
        for msg_idx in 0.. {
            let payload = generate_payload(producer_id, msg_idx as u32, opts.payload_size);
            trace!("Producer {} sends a message {}", producer_id, msg_idx);
            client
                .publish(&opts.topic, DEFAULT_QOS, false, payload)
                .await?;
        }
        anyhow::Ok(())
    };

    futures::try_join!(spin_worker, publish_worker)?;

    Ok(())
}

pub fn generate_payload(producer_id: u32, msg_index: u32, payload_size: usize) -> Vec<u8> {
    let producer_id_bytes: [u8; 4] = producer_id.to_le_bytes();
    let msg_idx_bytes: [u8; 4] = (msg_index as u32).to_le_bytes();
    let pad_bytes = {
        let pad_size = payload_size - producer_id_bytes.len() - msg_idx_bytes.len();
        iter::repeat(0u8).take(pad_size)
    };
    chain!(producer_id_bytes, msg_idx_bytes, pad_bytes).collect()
}
