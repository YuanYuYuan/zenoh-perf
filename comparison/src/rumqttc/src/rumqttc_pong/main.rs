mod opts;

use anyhow::{anyhow, Result};
use clap::Parser;
use log::{info, trace};
use opts::Opts;
use rumqttc::{Event, Packet};
use rumqttc_test::{create_client, DEFAULT_QOS};
use std::process;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let opts = Opts::parse();
    let timeout_dur = opts.timeout;
    let future = run_pong(opts);

    if let Some(timeout_dur) = timeout_dur {
        timeout(timeout_dur, future)
            .await
            .map_err(|_| anyhow!("timeout"))??;
    } else {
        future.await?;
    }

    Ok(())
}

async fn run_pong(opts: Opts) -> Result<()> {
    let pong_id = process::id();
    info!("Start pong {}", pong_id);

    let (client, mut event_loop) =
        create_client(env!("CARGO_BIN_NAME"), opts.broker, opts.max_payload_size)?;
    client.subscribe(&opts.ping_topic, DEFAULT_QOS).await?;

    loop {
        let event = event_loop.poll().await?;
        let Event::Incoming(Packet::Publish(publish)) =  event else {
            continue;
        };
        if publish.topic != opts.ping_topic {
            continue;
        }

        trace!("received a ping");
        client
            .publish(&opts.pong_topic, DEFAULT_QOS, false, publish.payload)
            .await?;
    }
}
