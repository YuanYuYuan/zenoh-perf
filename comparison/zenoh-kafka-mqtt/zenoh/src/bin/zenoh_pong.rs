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
use async_std::future;
use clap::Parser;
use zenoh::config::Config;
use zenoh::prelude::r#async::*;
use zenoh_protocol_core::{CongestionControl, EndPoint, WhatAmI};

#[derive(Debug, Parser)]
#[clap(name = "zenoh_pong")]
struct Opt {
    #[clap(short, long, value_delimiter = ',')]
    listen: Option<Vec<EndPoint>>,

    #[clap(short, long, value_delimiter = ',')]
    connect: Option<Vec<EndPoint>>,

    /// peer, router, or client
    #[clap(short, long, possible_values = ["client", "peer"])]
    mode: WhatAmI,

    /// declare key expression
    #[clap(long)]
    declare_keyexpr: bool,
}

const KEY_EXPR_PING: &str = "test/ping";
const KEY_EXPR_PONG: &str = "test/pong";

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
            if let Some(endpoints) = opt.listen {
                config.listen.endpoints.extend(endpoints)
            }
            if let Some(endpoints) = opt.connect {
                config.connect.endpoints.extend(endpoints)
            }
        }
        WhatAmI::Client => {
            if let Some(endpoints) = opt.connect {
                config.connect.endpoints.extend(endpoints)
            }
        }
        _ => panic!("Unsupported mode: {}", opt.mode),
    };
    config.scouting.multicast.set_enabled(Some(false)).unwrap();

    let session = zenoh::open(config).res().await.unwrap();
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
        .declare_publisher(key_expr_pong)
        .congestion_control(CongestionControl::Block)
        .res()
        .await
        .unwrap();
    let subscriber = session
        .declare_subscriber(key_expr_ping)
        .reliable()
        .res()
        .await
        .unwrap();

    while let Ok(sample) = subscriber.recv_async().await {
        publisher.put(sample).res().await.unwrap();
    }

    // Stop forever
    future::pending::<()>().await;
}
