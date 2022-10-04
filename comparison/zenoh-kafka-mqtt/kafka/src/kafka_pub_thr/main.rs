mod opts;

use std::{iter, process, time::Duration};

use anyhow::{anyhow, Result};
use clap::Parser;
use itertools::chain;
use kafka_test::AsyncStdFutureProducer;
use log::{info, trace};
use opts::Opts;
use rdkafka::{producer::FutureRecord, ClientConfig};

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

    // Configure the producer
    let mut client_config = ClientConfig::new();
    client_config.set("bootstrap.servers", &opts.brokers);

    if let Some(cfgs) = opts.producer_configs {
        cfgs.iter().for_each(|kv| {
            client_config.set(&kv.key, &kv.val);
        });
    }

    let producer: AsyncStdFutureProducer = client_config.create()?;

    // let producer: FutureProducer = ClientConfig::new()
    //     .set("bootstrap.servers", &opts.brokers)
    //     .set("message.timeout.ms", "5000")
    //     .set("linger.ms", "0")
    //     .set("batch.size", "200KB")
    //     .set("compression.type", "lz4")
    //     .set("acks", "1")
    //     // .set("metadata.max.age.ms", "180000")
    //     .create()?;

    for msg_idx in 0.. {
        let key = producer_id.to_le_bytes();
        let payload = generate_payload(producer_id, msg_idx as u32, opts.payload_size);
        let record = FutureRecord::to(&opts.topic).payload(&payload).key(&key);

        trace!("Producer {} sends a message {}", producer_id, msg_idx);
        producer
            .send(record, Duration::ZERO)
            .await
            .map_err(|(err, _msg)| err)?;
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
