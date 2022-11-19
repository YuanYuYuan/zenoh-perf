mod opts;

use anyhow::{anyhow, Result};
use clap::Parser;
use itertools::chain;
use log::{info, trace};
use opts::Opts;
use rumqttc::{AsyncClient, MqttOptions};
use rumqttc_test::{DEFAULT_CAP_SIZE, DEFAULT_QOS};
use std::{iter, process};

#[async_std::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let opts = Opts::parse();
    let timeout = opts.timeout;

    let future = run_producer(opts);
    if let Some(timeout) = timeout {
        async_std::future::timeout(timeout, future)
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
    let client_config = MqttOptions::new(
        env!("CARGO_PKG_NAME"),
        opts.broker.ip().to_string(),
        opts.broker.port(),
    );
    let (client, _event_loop) = AsyncClient::new(client_config, DEFAULT_CAP_SIZE);
    client.subscribe(&opts.topic, DEFAULT_QOS).await?;

    for msg_idx in 0.. {
        let payload = generate_payload(producer_id, msg_idx as u32, opts.payload_size);
        trace!("Producer {} sends a message {}", producer_id, msg_idx);
        client
            .publish(&opts.topic, DEFAULT_QOS, false, payload)
            .await?;
    }

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
