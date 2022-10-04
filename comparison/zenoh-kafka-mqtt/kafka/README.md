# Kafka Throughput & Lantency Benchmark

## Prerequisites

The Kafka and ZooKeeper must be started before running the
benchmark. This directory ships `scripts/install-kafka.sh` that can
download Kafka binaries.


Kafka has single server mode and cluster mode. To run in single server
mode, run this command and provide the server address.

```bash
./scripts/run_kafka_single.sh 127.0.0.1
```

To run kafka in cluster mode, you must modify `run_kafka_cluster.sh`
first before executing the script. See the [Configure the Kafka
Cluster](#configure-the-kafka-cluster) section below.

If you prefer to configure Kafka and ZooKeeper manually, Ubuntu 20.04
users can follow these tutorials.

- https://www.digitalocean.com/community/tutorials/how-to-install-apache-kafka-on-ubuntu-20-04
- https://hevodata.com/blog/how-to-install-kafka-on-ubuntu/


## Files

It provides four binaries in a single Cargo workspace. Both
`kafka_ping` and `kafka_pong` are exectued to perform latency
benchmark, while `kafka_pub_thr` and `kafka_sub_thr` are executed to
perform throughput benchmark. The directory layout looks like this.

```
repo/
├── Cargo.toml
├── scripts/
│   ├── payload_size.txt
│   ├── run_latency.sh
│   ├── run_many_latency.sh
│   ├── run_many_throughput.sh
│   ├── run_throughput.sh
│   └── ...
└── src/
    ├── lib.rs
    ├── kafka_ping/
    ├── kafka_pong/
    ├── kafka_pub_thr/
    └── kafka_sub_thr/
```

Among the files above,

- `src` stores the source of respective binaries.
- `script` provides the scripts to execute the latency and throughput
  benchmarks.


## Latency Benchmark

Run this script to execute many latency tests under various
configurations.

```sh
cd scrtips
./run_many_latency.sh
```

After this test, the program output files are created in the directory
`20XX-XX-XXTXX-XX-XX+XX-XX_latency`. In this directory, the
configuration of payload size X bytes and interval Y seconds is named
`payload-X_interval-Y`. The output directory looks like this.

```
2022-07-28T11-00-01+08-00_latency
├── payload-16_interval-0.001
│   ├── 1_cargo run --bin kafka_ping --release -- -p 16 -i 0
│   ├── 1_cargo run --bin kafka_ping --release -- -p 16 -i 0.err
│   ├── 1_cargo run --bin kafka_ping --release -- -p 16 -i 0.seq
│   ├── 2_cargo run --bin kafka_pong --release --
│   ├── 2_cargo run --bin kafka_pong --release --.err
│   └── 2_cargo run --bin kafka_pong --release --.seq
├── payload-16_interval-0.1
│   ├── 1_cargo run --bin kafka_ping --release -- -p 16 -i 0
│   ├── 1_cargo run --bin kafka_ping --release -- -p 16 -i 0.err
│   ├── 1_cargo run --bin kafka_ping --release -- -p 16 -i 0.seq
│   ├── 2_cargo run --bin kafka_pong --release --
│   ├── 2_cargo run --bin kafka_pong --release --.err
│   └── 2_cargo run --bin kafka_pong --release --.seq
```

Read the stdout of `kafka_ping`, the output looks like this.

```
# cat 2022-07-28T11-00-01+08-00_latency/payload-16_interval-0.001/1_cargo run --bin kafka_ping --release -- -p 16 -i 0
16 bytes: seq=0 time=6044049µs
16 bytes: seq=1 time=12526µs
16 bytes: seq=2 time=12472µs
16 bytes: seq=3 time=12555µs
16 bytes: seq=4 time=12544µs
...
```

## Throughput Benchmark


Run this script to execute many latency tests under various
configurations.

```sh
cd scrtips
./run_many_throughput.sh
```

After this test, the program output files are created in the directory
`20XX-XX-XXTXX-XX-XX+XX-XX_throughput`. In this directory, the
configuration of payload size X bytes and Y publishers is named
`payload-X_num-pub-Y`. The output directory looks like this.

```
2022-07-28T11-00-34+08-00_throughput
├── payload-16_num-pub-1
│   ├── 1_cargo run --bin kafka_sub_thr --release -- -p 16 -n 128
│   ├── 1_cargo run --bin kafka_sub_thr --release -- -p 16 -n 128.err
│   ├── 1_cargo run --bin kafka_sub_thr --release -- -p 16 -n 128.seq
│   ├── 2_cargo run --bin kafka_sub_thr --release -- -p 16
│   ├── 2_cargo run --bin kafka_sub_thr --release -- -p 16.err
│   └── 2_cargo run --bin kafka_sub_thr --release -- -p 16.seq
├── payload-16_num-pub-2
│   ├── 1_cargo run --bin kafka_sub_thr --release -- -p 16 -n 128
│   ├── 1_cargo run --bin kafka_sub_thr --release -- -p 16 -n 128.err
│   ├── 1_cargo run --bin kafka_sub_thr --release -- -p 16 -n 128.seq
│   ├── 2_cargo run --bin kafka_sub_thr --release -- -p 16
│   ├── 2_cargo run --bin kafka_sub_thr --release -- -p 16.err
│   ├── 2_cargo run --bin kafka_sub_thr --release -- -p 16.seq
│   ├── 3_cargo run --bin kafka_sub_thr --release -- -p 16
│   ├── 3_cargo run --bin kafka_sub_thr --release -- -p 16.err
│   └── 3_cargo run --bin kafka_sub_thr --release -- -p 16.seq
├── ...
```

Read the stdout of `kafka_sub_thr`. The output is a CSV file, in which
the last two columes are the payload size and the measured number of messsages
per seconds.

```
# cat 2022-07-28T11-00-34+08-00_throughput/payload-16_num-pub-1/1_cargo run --bin kafka_sub_thr --release -- -p 16 -n 128
kafka,DUMMY_SCENARIO,throughput,DUMMY_NAME,16,163.537
kafka,DUMMY_SCENARIO,throughput,DUMMY_NAME,16,165.258
kafka,DUMMY_SCENARIO,throughput,DUMMY_NAME,16,164.327
kafka,DUMMY_SCENARIO,throughput,DUMMY_NAME,16,171.431
```

## Command Line Options

For the latency test, in the following comand line, the `PAYLOAD_SIZE` is a size in bytes. The
`INTERVAL` is a floating point interval between consecutive pings in
seconds. The `COUNT` is the maximum number of pings to be sent.


```sh
cargo run --bin kafka_ping -- --p PAYLOAD_SIZE [-i INTERVAL] [-n COUNT]
cargo run --bin kafka_pong
```

For the throughput test, the `PAYLOAD_SIZE` is a size in bytes


```sh
cargo run --bin kafka_pub_thr -- -p PAYLOAD_SIZE
cargo run --bin kafka_pong
```

## Benchmark Configurations

The binaries assuem a Kafka broker at `127.0.0.1`. If it needs
to be changed, specify `-b COMMNA_SEPERATED_BROKERS` option in the command
line.

The througput benchmark code runs a consumer and one or more
publishers. They publish to or subscribe to the `THROUGHPUT` topic and
the 0th partition number.

The latency benchmark code runs a ping and a pong programs. The ping
messages go through the `PING` topic, while the pong messages go
through the `PONG` topic.

In both tests, default Kafka configurations are used. There is no time or
size retention set on the mentioned topics.

## Configure the Kafka Cluster

The cluster mode requires 3 participated Zookeepers at minimum, each
having an unique ID.

Suppose you have three servers: 10.8.0.251, 10.8.0.252 and 10.8.0.253,
assigned with IDs 1, 2 and 3. For the server 10.8.0.253, you have to
open `scripts/run_kafka_cluster.sh` and modify the variables like
this.

```sh
export MY_ID=3
export ADDRESS=10.8.0.253
export SERVER1=10.8.0.251
export SERVER1=10.8.0.252
export SERVER1=10.8.0.253
```
