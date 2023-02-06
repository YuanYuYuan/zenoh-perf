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
use std::collections::HashMap;
use std::sync::{Arc, Barrier, Mutex};
use std::time::Instant;
use structopt::StructOpt;
use zenoh::buffers::ZBuf;
use zenoh::prelude::SplitBuffer;
use zenoh::runtime::Runtime;
use zenoh_link::EndPoint;
use zenoh_protocol::proto::{DataInfo, QueryBody, RoutingContext};
use zenoh_protocol_core::{
    Channel, CongestionControl, ConsolidationMode, QueryTarget, QueryableInfo, SubInfo, WhatAmI,
    WireExpr, ZInt, ZenohId,
};
use zenoh_transport::Primitives;

type Pending = Arc<Mutex<HashMap<u64, (Instant, Arc<Barrier>)>>>;

struct QueryPrimitives {
    scenario: String,
    name: String,
    pending: Pending,
}

impl QueryPrimitives {
    pub fn new(scenario: String, name: String, pending: Pending) -> QueryPrimitives {
        QueryPrimitives {
            scenario,
            name,
            pending,
        }
    }
}

impl Primitives for QueryPrimitives {
    fn decl_resource(&self, _rid: ZInt, _key_expr: &WireExpr) {}
    fn forget_resource(&self, _rid: ZInt) {}
    fn decl_publisher(&self, _key_expr: &WireExpr, _routing_context: Option<RoutingContext>) {}
    fn forget_publisher(&self, _key_expr: &WireExpr, _routing_context: Option<RoutingContext>) {}
    fn decl_subscriber(
        &self,
        _key_expr: &WireExpr,
        _sub_info: &SubInfo,
        _routing_context: Option<RoutingContext>,
    ) {
    }
    fn forget_subscriber(&self, _key_expr: &WireExpr, _routing_context: Option<RoutingContext>) {}
    fn decl_queryable(
        &self,
        _key_expr: &WireExpr,
        _qabl_info: &QueryableInfo,
        _routing_context: Option<RoutingContext>,
    ) {
    }
    fn forget_queryable(&self, _key_expr: &WireExpr, _routing_context: Option<RoutingContext>) {}

    fn send_data(
        &self,
        _key_expr: &WireExpr,
        _payload: ZBuf,
        _channel: Channel,
        _congestion_control: CongestionControl,
        _data_info: Option<DataInfo>,
        _routing_context: Option<RoutingContext>,
    ) {
    }
    fn send_query(
        &self,
        _key_expr: &WireExpr,
        _parameters: &str,
        _qid: ZInt,
        _target: QueryTarget,
        _consolidation: ConsolidationMode,
        _body: Option<QueryBody>,
        _routing_context: Option<RoutingContext>,
    ) {
    }
    fn send_reply_data(
        &self,
        qid: ZInt,
        _replier_id: ZenohId,
        _key_expr: WireExpr,
        _info: Option<DataInfo>,
        payload: ZBuf,
    ) {
        let tuple = self.pending.lock().unwrap().remove(&qid).unwrap();
        let (instant, barrier) = (tuple.0, tuple.1);
        barrier.wait();
        println!(
            "router,{},query.latency,{},{},{},{}",
            self.scenario,
            self.name,
            payload.len(),
            qid,
            instant.elapsed().as_micros()
        );
    }
    fn send_reply_final(&self, _qid: ZInt) {}
    fn send_pull(
        &self,
        _is_final: bool,
        _key_expr: &WireExpr,
        _pull_id: ZInt,
        _max_samples: &Option<ZInt>,
    ) {
    }
    fn send_close(&self) {}
}

#[derive(Debug, StructOpt)]
#[structopt(name = "r_query")]
struct Opt {
    #[structopt(short = "l", long = "locator", value_delimiter = ",")]
    locator: Vec<EndPoint>,
    #[structopt(short = "m", long = "mode")]
    mode: WhatAmI,
    #[structopt(short = "n", long = "name")]
    name: String,
    #[structopt(short = "s", long = "scenario")]
    scenario: String,
}

#[async_std::main]
async fn main() {
    // Enable logging
    env_logger::init();

    // Parse the args
    let opt = Opt::from_args();

    let mut config = zenoh::prelude::Config::default();
    config.set_mode(Some(opt.mode)).unwrap();
    config.scouting.multicast.set_enabled(Some(false)).unwrap();
    config.connect.endpoints.extend(opt.locator);

    let pending: Pending = Arc::new(Mutex::new(HashMap::new()));

    let runtime = Runtime::new(config).await.unwrap();
    let rx_primitives = Arc::new(QueryPrimitives::new(
        opt.scenario,
        opt.name,
        pending.clone(),
    ));
    let tx_primitives = runtime.router.new_primitives(rx_primitives);

    let barrier = Arc::new(Barrier::new(2));
    let mut count: u64 = 0;
    loop {
        let key_expr = WireExpr::from("/test/query");
        let parameters = "";
        let qid = count;
        let target = QueryTarget::default();
        let consolidation = ConsolidationMode::None;
        let body = None;
        let routing_context = None;

        // Insert the pending query
        pending
            .lock()
            .unwrap()
            .insert(count, (Instant::now(), barrier.clone()));
        tx_primitives.send_query(
            &key_expr,
            parameters,
            qid,
            target,
            consolidation,
            body,
            routing_context,
        );
        // Wait for the reply to arrive
        barrier.wait();

        count += 1;
    }
}
