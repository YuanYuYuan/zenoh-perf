use anyhow::Result;
use clap::Parser;
use rumqttc_test::{DEFAULT_PING_TOPIC, DEFAULT_PONG_TOPIC};
use std::{net::SocketAddr, time::Duration};

#[derive(Parser)]
pub struct Opts {
    #[clap(long, default_value_t = DEFAULT_PING_TOPIC.to_string())]
    pub ping_topic: String,

    #[clap(long, default_value_t = DEFAULT_PONG_TOPIC.to_string())]
    pub pong_topic: String,

    #[clap(long, parse(try_from_str = parse_duration))]
    pub timeout: Option<Duration>,

    #[clap(short = 'b', long, default_value = "127.0.0.1:1883")]
    pub broker: SocketAddr,

    #[clap(long, default_value = "2147483648")]
    pub max_payload_size: usize,
}

fn parse_duration(text: &str) -> Result<Duration> {
    let dur = humantime::parse_duration(text)?;
    Ok(dur)
}
