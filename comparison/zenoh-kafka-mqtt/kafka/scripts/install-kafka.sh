#!/usr/bin/env bash
set -e

script_dir="$(cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd)"
mkdir -p "$script_dir/files"
cd "$script_dir/files"

# Download Kafka
wget --no-clobber 'https://dlcdn.apache.org/kafka/3.3.1/kafka_2.12-3.3.1.tgz'
tar --overwrite -xf kafka_2.13-3.3.1.tgz
mv --no-clobber kafka_2.13-3.3.1 kafka-pkg
