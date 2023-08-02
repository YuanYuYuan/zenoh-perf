#!/usr/bin/env bash

[ "$EUID" -eq 0 ] || {
    echo "Please run as root!"
    exit
}
LEVEL=-20

# LEVEL=0

PAYLOAD=8

parallel --lb <<EOF
# nice -n $LEVEL taskset -c 0,1 ./target/release/t_sink_tcp -l 127.0.0.1:7447
nice -n $LEVEL taskset -c 0,1 ./target/release/t_sub_thr -e tcp/127.0.0.1:7447 -m peer -p $PAYLOAD --name test --scenario test
# nice -n $LEVEL taskset -c 2,3 ./target/release/t_pub_thr -e tcp/127.0.0.1:7447 --mode peer --payload $PAYLOAD --send
nice -n $LEVEL taskset -c 2,3 ./target/release/t_pub_thr -e tcp/127.0.0.1:7447 --mode peer --payload $PAYLOAD
# nice -n $LEVEL taskset -c 0,1 ./target/debug/t_sub_thr -e tcp/127.0.0.1:7447 -m peer -p $PAYLOAD --name test --scenario test
# nice -n $LEVEL taskset -c 2,3 ./target/debug/t_pub_thr -e tcp/127.0.0.1:7447 --mode peer --payload $PAYLOAD
EOF
