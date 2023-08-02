#!/usr/bin/env bash
taskset -c 2,3 ./target/release/t_pub_thr -e tcp/127.0.0.1:7447 --mode peer --payload 8
