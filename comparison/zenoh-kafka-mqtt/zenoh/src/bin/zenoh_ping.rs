//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
use async_std::sync::{Arc, Mutex};
use async_std::task;
use clap::Parser;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use zenoh::config::Config;
use zenoh::prelude::r#async::*;
use zenoh_buffers::reader::{HasReader, Reader};
use zenoh_protocol_core::{CongestionControl, EndPoint, WhatAmI};

#[derive(Debug, Parser)]
#[clap(name = "zenoh_ping")]
struct Opt {
    #[clap(short, long, value_delimiter = ',')]
    listen: Option<Vec<EndPoint>>,

    #[clap(short, long, value_delimiter = ',')]
    connect: Option<Vec<EndPoint>>,

    /// peer, router, or client
    #[clap(short, long, possible_values = ["client", "peer"])]
    mode: WhatAmI,

    /// payload size (bytes)
    #[clap(short, long)]
    payload: usize,

    /// interval of sending message (sec)
    #[clap(short, long)]
    interval: f64,

    /// spawn a task to receive or not
    #[clap(long = "parallel")]
    parallel: bool,

    /// declare key expression
    #[clap(long)]
    declare_keyexpr: bool,
}

const KEY_EXPR_PING: &str = "test/ping";
const KEY_EXPR_PONG: &str = "test/pong";

async fn parallel(opt: Opt, config: Config) {
    let session = zenoh::open(config).res().await.unwrap();
    let session = Arc::new(session);

    // The hashmap with the pings
    let pending = Arc::new(Mutex::new(HashMap::<u64, Instant>::new()));

    let c_pending = pending.clone();
    let interval = opt.interval;

    let (key_expr_ping, key_expr_pong) = if opt.declare_keyexpr {
        (
            session.declare_keyexpr(KEY_EXPR_PING).res().await.unwrap(),
            session.declare_keyexpr(KEY_EXPR_PONG).res().await.unwrap(),
        )
    } else {
        (
            zenoh::key_expr::KeyExpr::new(KEY_EXPR_PING).unwrap(),
            zenoh::key_expr::KeyExpr::new(KEY_EXPR_PONG).unwrap(),
        )
    };
    let publisher = session
        .declare_publisher(key_expr_ping)
        .congestion_control(CongestionControl::Block)
        .res()
        .await
        .unwrap();
    let subscriber = session
        .declare_subscriber(key_expr_pong)
        .reliable()
        .res()
        .await
        .unwrap();

    task::spawn(async move {
        while let Ok(sample) = subscriber.recv_async().await {
            let mut payload_reader = sample.value.payload.reader();
            let mut count_bytes = [0u8; 8];
            if payload_reader.read_exact(&mut count_bytes) {
                let count = u64::from_le_bytes(count_bytes);

                let instant = c_pending.lock().await.remove(&count).unwrap();
                println!("{},{}", interval, instant.elapsed().as_micros() / 2);
            } else {
                panic!("Fail to fill the buffer");
            }
        }
        panic!("Invalid value!");
    });

    let mut count: u64 = 0;
    loop {
        let count_bytes: [u8; 8] = count.to_le_bytes();
        let mut payload = vec![0u8; opt.payload];
        payload[0..8].copy_from_slice(&count_bytes);

        pending.lock().await.insert(count, Instant::now());

        publisher.put(payload).res().await.unwrap();

        task::sleep(Duration::from_secs_f64(opt.interval)).await;
        count += 1;
    }
}

async fn single(opt: Opt, config: Config) {
    let session = zenoh::open(config).res().await.unwrap();

    let interval = opt.interval;

    let (key_expr_ping, key_expr_pong) = if opt.declare_keyexpr {
        (
            session.declare_keyexpr(KEY_EXPR_PING).res().await.unwrap(),
            session.declare_keyexpr(KEY_EXPR_PONG).res().await.unwrap(),
        )
    } else {
        (
            zenoh::key_expr::KeyExpr::new(KEY_EXPR_PING).unwrap(),
            zenoh::key_expr::KeyExpr::new(KEY_EXPR_PONG).unwrap(),
        )
    };
    let publisher = session
        .declare_publisher(key_expr_ping)
        .congestion_control(CongestionControl::Block)
        .res()
        .await
        .unwrap();
    let subscriber = session
        .declare_subscriber(key_expr_pong)
        .reliable()
        .res()
        .await
        .unwrap();

    let mut count: u64 = 0;
    loop {
        let count_bytes: [u8; 8] = count.to_le_bytes();
        let mut payload = vec![0u8; opt.payload];
        payload[0..8].copy_from_slice(&count_bytes);

        let now = Instant::now();
        publisher.put(payload).res().await.unwrap();
        match subscriber.recv_async().await {
            Ok(sample) => {
                let mut payload_reader = sample.value.payload.reader();
                let mut count_bytes = [0u8; 8];
                if payload_reader.read_exact(&mut count_bytes) {
                    println!("{},{}", interval, now.elapsed().as_micros() / 2);
                } else {
                    panic!("Fail to fill the buffer");
                }
            }
            _ => panic!("Invalid value"),
        }
        task::sleep(Duration::from_secs_f64(opt.interval)).await;
        count += 1;
    }
}

#[async_std::main]
async fn main() {
    // initiate logging
    env_logger::init();

    // Parse the args
    let opt = Opt::parse();

    let mut config = Config::default();
    config.set_mode(Some(opt.mode)).unwrap();
    match opt.mode {
        WhatAmI::Peer => {
            if let Some(endpoints) = opt.listen.clone() {
                config.listen.endpoints.extend(endpoints)
            }
            if let Some(endpoints) = opt.connect.clone() {
                config.connect.endpoints.extend(endpoints)
            }
        }
        WhatAmI::Client => {
            if let Some(endpoints) = opt.connect.clone() {
                config.connect.endpoints.extend(endpoints)
            }
        }
        _ => panic!("Unsupported mode: {}", opt.mode),
    };
    config.scouting.multicast.set_enabled(Some(false)).unwrap();

    if opt.parallel {
        parallel(opt, config).await;
    } else {
        single(opt, config).await;
    }
}
