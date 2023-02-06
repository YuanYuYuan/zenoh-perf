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
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Barrier, Mutex};
use std::time::Instant;
use structopt::StructOpt;
use zenoh::net::link::{EndPoint, Link};
use zenoh::net::protocol::core::{whatami, ConsolidationMode, QueryTarget, ResKey};
use zenoh::net::protocol::proto::{Data, ZenohBody, ZenohMessage};
use zenoh::net::transport::*;
use zenoh_util::core::ZResult;
use zenoh_util::properties::{IntKeyProperties, Properties};

type Pending = Arc<Mutex<HashMap<u64, (Instant, Arc<Barrier>)>>>;

// Transport Handler for the blocking locator
struct MySH {
    scenario: String,
    name: String,
    pending: Pending,
}

impl MySH {
    fn new(scenario: String, name: String, pending: Pending) -> Self {
        Self {
            scenario,
            name,
            pending,
        }
    }
}

impl TransportEventHandler for MySH {
    fn new_unicast(
        &self,
        _peer: TransportPeer,
        _transport: TransportUnicast,
    ) -> ZResult<Arc<dyn TransportPeerEventHandler>> {
        Ok(Arc::new(MyMH::new(
            self.scenario.clone(),
            self.name.clone(),
            self.pending.clone(),
        )))
    }

    fn new_multicast(
        &self,
        _transport: TransportMulticast,
    ) -> ZResult<Arc<dyn TransportMulticastEventHandler>> {
        panic!();
    }
}

// Message Handler for the locator
struct MyMH {
    scenario: String,
    name: String,
    pending: Pending,
}

impl MyMH {
    fn new(scenario: String, name: String, pending: Pending) -> Self {
        Self {
            scenario,
            name,
            pending,
        }
    }
}

impl TransportPeerEventHandler for MyMH {
    fn handle_message(&self, message: ZenohMessage) -> ZResult<()> {
        match message.body {
            ZenohBody::Data(Data {
                payload,
                reply_context,
                ..
            }) => {
                let reply_context = reply_context.unwrap();
                let tuple = self
                    .pending
                    .lock()
                    .unwrap()
                    .remove(&reply_context.qid)
                    .unwrap();
                let (instant, barrier) = (tuple.0, tuple.1);
                barrier.wait();
                println!(
                    "session,{},query.latency,{},{},{},{}",
                    self.scenario,
                    self.name,
                    payload.len(),
                    reply_context.qid,
                    instant.elapsed().as_micros()
                );
            }
            _ => panic!("Invalid message"),
        }
        Ok(())
    }

    fn new_link(&self, _link: Link) {}
    fn del_link(&self, _link: Link) {}
    fn closing(&self) {}
    fn closed(&self) {}
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "s_query")]
struct Opt {
    #[structopt(short = "l", long = "locator")]
    locator: EndPoint,
    #[structopt(short = "m", long = "mode")]
    mode: String,
    #[structopt(short = "n", long = "name")]
    name: String,
    #[structopt(short = "s", long = "scenario")]
    scenario: String,
    #[structopt(long = "conf", parse(from_os_str))]
    config: Option<PathBuf>,
}

#[async_std::main]
async fn main() {
    // Enable logging
    env_logger::init();

    // Parse the args
    let opt = Opt::from_args();

    let whatami = whatami::parse(opt.mode.as_str()).unwrap();

    let pending: Pending = Arc::new(Mutex::new(HashMap::new()));

    let bc = match opt.config.as_ref() {
        Some(f) => {
            let config = async_std::fs::read_to_string(f).await.unwrap();
            let properties = Properties::from(config);
            let int_props = IntKeyProperties::from(properties);
            TransportManagerConfig::builder()
                .from_config(&int_props)
                .await
                .unwrap()
        }
        None => TransportManagerConfig::builder().whatami(whatami),
    };
    let config = bc.build(Arc::new(MySH::new(
        opt.scenario.clone(),
        opt.name.clone(),
        pending.clone(),
    )));
    let manager = TransportManager::new(config);

    // Connect to publisher
    let session = manager.open_transport(opt.locator).await.unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let mut count: u64 = 0;
    loop {
        // Create and send the message
        let key = ResKey::RName("/test/query".to_string());
        let predicate = "".to_string();
        let qid = count;
        let target = Some(QueryTarget::default());
        let consolidation = ConsolidationMode::default();
        let routing_context = None;
        let attachment = None;

        let message = ZenohMessage::make_query(
            key,
            predicate,
            qid,
            target,
            consolidation,
            routing_context,
            attachment,
        );

        // Insert the pending query
        pending
            .lock()
            .unwrap()
            .insert(count, (Instant::now(), barrier.clone()));
        session.handle_message(message).unwrap();
        // Wait for the reply to arrive
        barrier.wait();

        count += 1;
    }
}
