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
use clap::Parser;
use std::{
    path::PathBuf,
};
use std::thread;
use zenoh::prelude::{sync::*, CongestionControl};
use zenoh::{config::{Config, EndPoint, WhatAmI}, prelude::Value};

#[derive(Debug, Parser)]
#[clap(name = "zenoh_pub_thr")]
struct Opt {
    #[clap(short, long, value_delimiter = ',')]
    listen: Option<Vec<EndPoint>>,

    #[clap(short, long, value_delimiter = ',')]
    connect: Option<Vec<EndPoint>>,

    /// peer, router, or client
    #[clap(short, long)]
    mode: WhatAmI,

    /// payload size (bytes)
    #[clap(short, long)]
    payload: usize,

    /// configuration file (json5 or yaml)
    #[clap(long = "conf", value_parser)]
    config: Option<PathBuf>,

    /// number of duplications
    #[clap(short, long, default_value="1")]
    num: usize,
}

const KEY_EXPR: &str = "test/thr";

fn main() {
    // Initiate logging
    env_logger::init();

    // Parse the args
    let Opt {
        listen,
        connect,
        mode,
        payload,
        config,
        num,
    } = Opt::parse();
    let config = {
        let mut config: Config = if let Some(path) = config {
            Config::from_file(path).unwrap()
        } else {
            Config::default()
        };
        config.set_mode(Some(mode)).unwrap();
        config
            .timestamping
            .set_enabled(Some(zenoh::config::ModeDependentValue::Unique(false)))
            .unwrap();
        config.scouting.multicast.set_enabled(Some(false)).unwrap();
        config.scouting.gossip.set_enabled(Some(false)).unwrap();
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

    let data: Value = (0usize..payload)
        .map(|i| (i % 10) as u8)
        .collect::<Vec<u8>>()
        .into();

    let session = zenoh::open(config).res().unwrap().into_arc();

    for _ in 0..num {
        let data = data.clone();
        let publisher = session
            .clone()
            .declare_publisher(KEY_EXPR)
            .congestion_control(CongestionControl::Block)
            .res()
            .unwrap();
        thread::spawn(move || loop {
            publisher.put(data.clone()).res().unwrap();
        });
    }

    std::thread::park();

}
