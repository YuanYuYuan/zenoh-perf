mod opts;

use anyhow::{anyhow, ensure, Result};
use clap::Parser;
use log::{error, info, trace};
use opts::Opts;
use rumqttc::{Event, Packet};
use rumqttc_test::{create_client, DEFAULT_QOS};
use std::{
    process,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{
    spawn,
    time::{sleep, timeout},
};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let opts = Opts::parse();
    let timeout_dur = opts.timeout;

    let future = run_consumer(opts);
    if let Some(timeout_dur) = timeout_dur {
        timeout(timeout_dur, future)
            .await
            .map_err(|_| anyhow!("timeout"))??;
    } else {
        future.await?;
    }

    Ok(())
}

async fn run_consumer(opts: Opts) -> Result<()> {
    let consumer_id = process::id();
    info!("Start consumer {} on topic {}", consumer_id, opts.topic);

    // Configure the client
    let (client, mut event_loop) =
        create_client(env!("CARGO_BIN_NAME"), opts.broker, opts.payload_size)?;
    client.subscribe(&opts.topic, DEFAULT_QOS).await?;

    let counter = Arc::new(AtomicUsize::new(0));
    spawn(measure(counter.clone(), opts.payload_size));

    loop {
        let event = event_loop.poll().await?;
        let Event::Incoming(Packet::Publish(publish)) = event else {
            continue;
        };

        if publish.topic != opts.topic {
            continue;
        }

        // decode the producer ID and message index in the payload
        let info = match parse_payload(&publish.payload, opts.payload_size) {
            Ok(info) => info,
            Err(err) => {
                error!("Unable to parse payload: {:?}", err);
                continue;
            }
        };
        trace!(
            "Consumer {} receives a payload with index {} and size {} from producer {}",
            consumer_id,
            info.msg_index,
            publish.payload.len(),
            info.producer_id
        );
        counter.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn parse_payload(payload: &[u8], expect_size: usize) -> Result<PayloadInfo> {
    let payload_size = payload.len();
    ensure!(
        payload_size >= 8,
        "insufficient payload size {}",
        payload_size
    );
    ensure!(
        payload_size == expect_size,
        "payload size does not match, expect {} bytes, but received {} bytes",
        expect_size,
        payload_size
    );

    let producer_id = u32::from_le_bytes(payload[0..4].try_into().unwrap());
    let msg_index = u32::from_le_bytes(payload[4..8].try_into().unwrap());

    Ok(PayloadInfo {
        producer_id,
        msg_index,
        payload_size,
    })
}

#[derive(Debug, Clone)]
pub struct PayloadInfo {
    pub producer_id: u32,
    pub msg_index: u32,
    pub payload_size: usize,
}

async fn measure(messages: Arc<AtomicUsize>, payload: usize) {
    let mut timer = Instant::now();
    loop {
        sleep(Duration::from_secs(1)).await;

        if messages.load(Ordering::Relaxed) > 0 {
            let elapsed = timer.elapsed().as_micros() as f64;
            let c = messages.swap(0, Ordering::Relaxed);
            println!("{},{:.3}", payload, c as f64 * 1_000_000.0 / elapsed);
            timer = Instant::now()
        }
    }
}
