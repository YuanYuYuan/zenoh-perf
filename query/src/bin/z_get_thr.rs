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
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use structopt::StructOpt;
use zenoh::prelude::r#async::*;
use zenoh_link::EndPoint;
use zenoh_protocol_core::WhatAmI;

#[derive(Debug, StructOpt)]
#[structopt(name = "z_get_thr")]
struct Opt {
    #[structopt(short = "l", long = "locator")]
    locator: Vec<EndPoint>,
    #[structopt(short = "m", long = "mode")]
    mode: WhatAmI,
    #[structopt(short = "n", long = "name")]
    name: String,
    #[structopt(short = "s", long = "scenario")]
    scenario: String,
    #[structopt(short = "p", long = "payload")]
    payload: usize,
}

#[async_std::main]
async fn main() {
    // initiate logging
    env_logger::init();

    // Parse the args
    let opt = Opt::from_args();

    let mut config: Config = Config::default();
    config.set_mode(Some(opt.mode)).unwrap();
    config.scouting.multicast.set_enabled(Some(false)).unwrap();
    config.connect.endpoints.extend(opt.locator.clone());

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
                let interval = 1_000_000.0 / elapsed;
                println!(
                    "zenoh,{},query.throughput,{},{},{},{}",
                    opt.scenario,
                    opt.name,
                    opt.payload,
                    (c as f64 / interval).floor() as usize,
                    (r as f64 / c as f64).floor() as usize,
                );
            }
        }
    });

    loop {
        let now = Instant::now();
        let data_stream = session.get("/test/query").res().await.unwrap();
        while data_stream.recv_async().await.is_ok() {}

        rtt.fetch_add(now.elapsed().as_micros() as usize, Ordering::Relaxed);
        counter.fetch_add(1, Ordering::Relaxed);
    }
}
