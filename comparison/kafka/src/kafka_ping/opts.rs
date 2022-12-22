use anyhow::Result;
use clap::{Parser, ValueEnum};
use kafka_test::{KeyVal, DEFAULT_PING_TOPIC, DEFAULT_PONG_TOPIC};
use std::time::Duration;

#[derive(Parser)]
pub struct Opts {
    #[clap(long, default_value_t = DEFAULT_PING_TOPIC.to_string())]
    pub ping_topic: String,
    #[clap(long, default_value_t = DEFAULT_PONG_TOPIC.to_string())]
    pub pong_topic: String,
    #[clap(long, parse(try_from_str = parse_timeout))]
    pub timeout: Option<Duration>,
    #[clap(long, value_enum, default_value = "human")]
    pub output_format: OutputFormat,

    #[clap(short = 'b', long, default_value = "127.0.0.1")]
    pub brokers: String,

    #[clap(short, long, help = "ping interval in seconds")]
    pub interval: f64,

    #[clap(short, long)]
    pub payload_size: usize,

    #[clap(short = 'P', long)]
    pub producer_configs: Option<Vec<KeyVal>>,

    #[clap(short = 'C', long)]
    pub consumer_configs: Option<Vec<KeyVal>>,
}

fn parse_timeout(text: &str) -> Result<Duration> {
    let dur = humantime::parse_duration(text)?;
    Ok(dur)
}

// fn parse_interval(text: &str) -> Result<Duration> {
//     let secs = R64::from_str_radix(text, 10).map_err(|_| anyhow!("invalid interval {}", text))?;
//     let dur = Duration::from_secs_f64(secs.raw());
//     Ok(dur)
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum OutputFormat {
    Human,
    Json,
}
