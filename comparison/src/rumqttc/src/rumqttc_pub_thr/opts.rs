use anyhow::Result;
use clap::{Parser, ValueEnum};
use rumqttc_test::DEFAULT_THROUGHPUT_TOPIC;
use std::{net::SocketAddr, time::Duration};

#[derive(Parser)]
pub struct Opts {
    #[clap(long, default_value_t = DEFAULT_THROUGHPUT_TOPIC.to_string())]
    pub topic: String,
    #[clap(long, parse(try_from_str = parse_duration))]
    pub timeout: Option<Duration>,

    #[clap(short = 'b', long, default_value = "127.0.0.1:1883")]
    pub broker: SocketAddr,
    #[clap(short = 'p', long)]
    pub payload_size: usize,
    // #[clap(short = 'P', long)]
    // pub producer_configs: Option<Vec<KeyVal>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum Mode {
    Peer,
    Client,
}

fn parse_duration(text: &str) -> Result<Duration> {
    let dur = humantime::parse_duration(text)?;
    Ok(dur)
}
