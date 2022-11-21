use anyhow::Result;
use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use std::net::SocketAddr;

pub const DEFAULT_THROUGHPUT_TOPIC: &str = "THROUGHPUT";
pub const DEFAULT_PING_TOPIC: &str = "PING";
pub const DEFAULT_PONG_TOPIC: &str = "PONG";
pub const DEFAULT_GROUP_ID: &str = "DUMMY_GROUP";
pub const DEFAULT_QOS: QoS = QoS::AtMostOnce;
pub const DEFAULT_CAP_SIZE: usize = 10;
pub const PACKET_HEADER_SIZE: usize = 14;

pub fn create_client(
    name: &str,
    broker_addr: SocketAddr,
    max_payload_size: usize,
) -> Result<(AsyncClient, EventLoop)> {
    let mut client_config =
        MqttOptions::new(name, broker_addr.ip().to_string(), broker_addr.port());
    let max_packet_size = max_payload_size + PACKET_HEADER_SIZE;
    client_config.set_max_packet_size(max_packet_size, max_packet_size);
    let (client, event_loop) = AsyncClient::new(client_config, DEFAULT_CAP_SIZE);
    Ok((client, event_loop))
}
