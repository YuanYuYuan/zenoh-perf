mod opts;

use anyhow::{anyhow, ensure, Context, Result};
use clap::Parser;
use log::{error, info, trace};
use once_cell::sync::Lazy;
use opts::Opts;
use rumqttc::{AsyncClient, Event, EventLoop, Packet};
use rumqttc_test::{create_client, DEFAULT_QOS};
use std::{
    process,
    time::{Duration, SystemTime},
};
use tokio::time::{sleep, timeout};

static SINCE: Lazy<SystemTime> = Lazy::new(SystemTime::now);
const MIN_PAYLOAD_SIZE: usize = 16;

struct PayloadInfo {
    pub ping_id: u32,
    pub msg_idx: u32,
    pub rtt: Duration,
}

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let opts = Opts::parse();
    let timeout_dur = opts.timeout;

    let future = run_latency_benchmark(opts);
    if let Some(timeout_dur) = timeout_dur {
        timeout(timeout_dur, future)
            .await
            .map_err(|_| anyhow!("timeout"))??;
    } else {
        future.await?;
    }

    Ok(())
}

async fn run_latency_benchmark(opts: Opts) -> Result<()> {
    let ping_id = process::id();
    info!("Start ping {}", ping_id);

    let (client, mut event_loop) =
        create_client(env!("CARGO_BIN_NAME"), opts.broker, opts.payload_size)?;
    client.subscribe(&opts.pong_topic, DEFAULT_QOS).await?;

    for count in 0.. {
        send(&opts, &client, ping_id, count).await?;
        if !recv(&opts, ping_id, &mut event_loop).await? {
            panic!("Failed to receive pong message.");
        }
        sleep(Duration::from_secs_f64(opts.interval)).await;
    }

    Ok(())
}

fn generate_payload(size: usize, ping_id: u32, msg_idx: u32) -> Vec<u8> {
    assert!(
        size >= MIN_PAYLOAD_SIZE,
        "The minimum payload size is {} bytes",
        MIN_PAYLOAD_SIZE
    );
    let since = *SINCE; // the SINCE is inited earlier than the next SystemTime::now()
    let dur = SystemTime::now().duration_since(since).unwrap();
    let micros = dur.as_micros() as u64;

    let ping_id_bytes = ping_id.to_le_bytes();
    let msg_idx_bytes = msg_idx.to_le_bytes();
    let time_bytes = micros.to_le_bytes();

    let mut payload = vec![0u8; size];
    payload[0..4].copy_from_slice(&ping_id_bytes);
    payload[4..8].copy_from_slice(&msg_idx_bytes);
    payload[8..16].copy_from_slice(&time_bytes);
    payload
}

fn parse_payload(payload: &[u8], expect_payload_size: usize) -> Result<PayloadInfo> {
    let payload_size = payload.len();
    ensure!(
        payload_size >= MIN_PAYLOAD_SIZE,
        "The payload size ({} bytes) is less than the required minimum {} bytes",
        payload_size,
        MIN_PAYLOAD_SIZE
    );

    let ping_id_bytes = &payload[0..4];
    let msg_idx_bytes = &payload[4..8];
    let time_bytes = &payload[8..16];

    let micros = u64::from_le_bytes(time_bytes.try_into().unwrap());
    let since = *SINCE + Duration::from_micros(micros);
    let rtt = SystemTime::now()
        .duration_since(since)
        .with_context(|| "the timestamp goes backward")?;

    let ping_id = u32::from_le_bytes(ping_id_bytes.try_into().unwrap());
    let msg_idx = u32::from_le_bytes(msg_idx_bytes.try_into().unwrap());

    ensure!(
        payload.len() == expect_payload_size,
        "Expect payload size to be {} bytes, but get {} bytes",
        expect_payload_size,
        payload_size
    );

    Ok(PayloadInfo {
        msg_idx,
        rtt,
        ping_id,
    })
}

async fn send(opts: &Opts, client: &AsyncClient, ping_id: u32, msg_idx: u32) -> Result<()> {
    let payload = generate_payload(opts.payload_size, ping_id, msg_idx);

    trace!(
        "Send a ping with ping_id {} and msg_idx {}",
        ping_id,
        msg_idx
    );
    client
        .publish(&opts.ping_topic, DEFAULT_QOS, false, payload)
        .await?;
    Ok(())
}

async fn recv(opts: &Opts, ping_id: u32, event_loop: &mut EventLoop) -> Result<bool> {
    const RECV_TIMEOUT: Duration = Duration::from_secs(1);

    let publish = loop {
        let poll = event_loop.poll();
        let result = timeout(RECV_TIMEOUT, poll).await;
        let Ok(result) = result else {
            trace!("Timeout receiving a message");
            return Ok(true)

        };
        let event = result?;
        let Event::Incoming(Packet::Publish(publish)) = event else {
            continue;
        };

        if publish.topic != opts.pong_topic {
            continue;
        }

        break publish;
    };

    let info = match parse_payload(&publish.payload, opts.payload_size) {
        Ok(info) => info,
        Err(err) => {
            error!("Unable to parse payload: {:#}", err);
            return Ok(false);
        }
    };

    trace!(
        "Received a pong with ping_id {} and msg_idx {}",
        info.ping_id,
        info.msg_idx
    );
    ensure!(
        info.ping_id == ping_id,
        "Ignore the payload from a foreign ping ID {}",
        info.ping_id
    );

    println!("{},{}", opts.interval, info.rtt.as_micros() / 2);

    Ok(true)
}
