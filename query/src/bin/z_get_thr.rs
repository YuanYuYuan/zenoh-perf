//
// Copyright (c) 2017, 2020 ADLINK Technology Inc.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ADLINK zenoh team, <zenoh@adlink-labs.tech>
//
use async_std::sync::Arc;
use async_std::task;
use clap::Parser;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use zenoh::prelude::r#async::*;
use zenoh::Result;
use zenoh_protocol::core::{EndPoint, WhatAmI};

const KEY_EXPR: &str = "test/query";

#[derive(Debug, Parser)]
#[clap(name = "z_get_thr")]
struct Opt {
    #[clap(short, long, value_delimiter = ',')]
    listen: Option<Vec<EndPoint>>,

    #[clap(short, long, value_delimiter = ',')]
    connect: Option<Vec<EndPoint>>,

    #[clap(short, long)]
    mode: WhatAmI,

    #[clap(short, long)]
    name: Option<String>,

    #[clap(short, long)]
    scenario: Option<String>,

    #[clap(short, long)]
    payload: usize,

    /// configuration file (json5 or yaml)
    #[clap(long = "conf", value_parser)]
    config: Option<PathBuf>,
}

#[async_std::main]
async fn main() -> Result<()> {
    // initiate logging
    env_logger::init();

    // Parse the args
    // Parse the args
    let Opt {
        listen,
        connect,
        mode,
        name,
        scenario,
        payload,
        config,
    } = Opt::parse();

    let config = {
        let mut config: Config = if let Some(path) = config {
            Config::from_file(path).unwrap()
        } else {
            Config::default()
        };
        config.set_mode(Some(mode)).unwrap();
        config.scouting.multicast.set_enabled(Some(false)).unwrap();
        match mode {
            WhatAmI::Peer => {
                if let Some(endpoint) = listen {
                    config.listen.endpoints.extend(endpoint);
                }
                if let Some(endpoint) = connect {
                    config.connect.endpoints.extend(endpoint);
                }
            }
            WhatAmI::Client => {
                if let Some(endpoint) = connect {
                    config.connect.endpoints.extend(endpoint);
                }
            }
            _ => panic!("Unsupported mode: {mode}"),
        };
        config
    };

    let session = zenoh::open(config).res().await.unwrap();

    let rtt = Arc::new(AtomicUsize::new(0));
    let counter = Arc::new(AtomicUsize::new(0));

    let c_rtt = rtt.clone();
    let c_counter = counter.clone();
    task::spawn(async move {
        loop {
            let now = Instant::now();
            task::sleep(Duration::from_secs(1)).await;
            let elapsed = now.elapsed().as_micros() as f64;

            let r = c_rtt.swap(0, Ordering::Relaxed);
            let c = c_counter.swap(0, Ordering::Relaxed);
            if c > 0 {
                let interval = elapsed / 1_000_000.0;
                if let (Some(scenario), Some(name)) = (&scenario, &name) {
                    println!(
                        "zenoh,{},query.throughput,{},{},{},{}",
                        scenario,
                        name,
                        payload,
                        (c as f64 / interval).floor() as usize,
                        (r as f64 / c as f64).floor() as usize,
                    );
                } else {
                    println!(
                        "{},{},{}",
                        payload,
                        (c as f64 / interval).floor() as usize,
                        (r as f64 / c as f64).floor() as usize,
                    );
                }
            }
        }
    });

    loop {
        let now = Instant::now();
        let replies = session.get(KEY_EXPR).res().await.unwrap();
        while let Ok(reply) = replies.recv_async().await {
            assert_eq!(reply.sample?.value.payload.len(), payload);
            counter.fetch_add(1, Ordering::Relaxed);
        }
        rtt.fetch_add(now.elapsed().as_micros() as usize, Ordering::Relaxed);
    }
}
