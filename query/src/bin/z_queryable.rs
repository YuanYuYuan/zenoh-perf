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
use clap::Parser;
use std::path::PathBuf;
use zenoh::key_expr::KeyExpr;
use zenoh::prelude::r#async::*;
use zenoh::Result;
use zenoh::{config::Config, prelude::Sample};
use zenoh_protocol::core::{EndPoint, WhatAmI};

const KEY_EXPR: &str = "test/query";

#[derive(Debug, Parser)]
#[clap(name = "z_queryable")]
struct Opt {
    #[clap(short, long, value_delimiter = ',')]
    listen: Option<Vec<EndPoint>>,

    #[clap(short, long, value_delimiter = ',')]
    connect: Option<Vec<EndPoint>>,

    #[clap(short, long)]
    mode: WhatAmI,

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
    let Opt {
        listen,
        connect,
        mode,
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

    let session = zenoh::open(config).res().await?;
    let key_expr = KeyExpr::new(KEY_EXPR)?;
    let queryable = session.declare_queryable(&key_expr).res().await?;
    let data: Value = (0usize..payload)
        .map(|i| (i % 10) as u8)
        .collect::<Vec<u8>>()
        .into();
    while let Ok(query) = queryable.recv_async().await {
        query
            .reply(Ok(Sample::new(key_expr.clone(), data.clone())))
            .res()
            .await?;
    }
    Ok(())
}
