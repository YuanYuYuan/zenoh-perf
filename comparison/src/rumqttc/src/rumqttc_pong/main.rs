mod opts;

use anyhow::{anyhow, Result};
use clap::Parser;
use log::{info, trace};
use opts::Opts;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet};
use rumqttc_test::{DEFAULT_CAP_SIZE, DEFAULT_QOS};
use std::process;

#[async_std::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let opts = Opts::parse();
    let timeout = opts.timeout;
    let future = run_pong(opts);

    if let Some(timeout) = timeout {
        async_std::future::timeout(timeout, future)
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

    let client_config = MqttOptions::new(
        env!("CARGO_PKG_NAME"),
        opts.broker.ip().to_string(),
        opts.broker.port(),
    );
    let (client, mut event_loop) = AsyncClient::new(client_config, DEFAULT_CAP_SIZE);
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
