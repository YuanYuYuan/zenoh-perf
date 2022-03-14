#!/usr/bin/env bash

PUB_P2P_NICE="-10"
PUB_P2P_CPUS="0,1"

SUB_P2P_NICE="-10"
SUB_P2P_CPUS="2,3"

TEST_CONF="myTestConfig"
TEST_TIME="15"
LOCALHOST="127.0.0.1"
LOCATOR_P2P="tcp/$LOCALHOST:7447"

if [ $# -ne 1 ]; then
    echo "Usage: ./p2p.sh CONFIG.sh"
    exit 1
fi

source $1
LOG_DIR="outputs/$ZENOH_PERF_VER/logs"
USAGE_DIR="outputs/$ZENOH_PERF_VER/usages"
PUB_USAGE_DIR="${USAGE_DIR}/pub"
SUB_USAGE_DIR="${USAGE_DIR}/sub"
mkdir -p $LOG_DIR
mkdir -p $PUB_USAGE_DIR
mkdir -p $SUB_USAGE_DIR

trap ctrl_c INT

function ctrl_c() {
    cleanup
    exit
}

function cleanup() {
    killall -9 $SUB_PROGRAM
    killall -9 $PUB_PROGRAM
}

PAYLOAD_LIST=(
    8
    16
    32
    64
    128
    256
    512
    1024
    2048
    4096
    8192
    16384
    32768
    65500
    128000
    256000
    512000
    1024000
    2048000
    4096000
)


# for P in {0..26}; do
#     PAYLOAD=$((8*2**P))
for PAYLOAD in ${PAYLOAD_LIST[@]}; do
    DATE=$(date +%Y%m%d.%H%M%S)
    LOG_FILE="${LOG_DIR}/${PAYLOAD}.log"

    echo
    echo "Testing $PAYLOAD bytes. ID: $DATE"
    echo 'layer,scenario,test,name,size,messages' > $LOG_FILE

    echo ">>> Starting zenoh test on SUB... $DATE"
    psrecord "
        nice $SUB_P2P_NICE taskset -c $SUB_P2P_CPUS \
        $ZENOH_PERF_DIR/target/release/$SUB_PROGRAM \
        --scenario $TEST_CONF \
        --mode peer \
        --locator $LOCATOR_P2P \
        --payload $PAYLOAD \
        --name $DATE >> $LOG_FILE
    " \
        --log ${SUB_USAGE_DIR}/${PAYLOAD}.txt \
        --include-children \
        --duration $TEST_TIME &


    sleep 5
    echo ">>> Starting zenoh test on PUB... $DATE"
    psrecord "
        nice $PUB_P2P_NICE taskset -c $PUB_P2P_CPUS timeout $TEST_TIME \
            $ZENOH_PERF_DIR/target/release/$PUB_PROGRAM \
            --mode peer \
            --locator $LOCATOR_P2P \
            --payload $PAYLOAD
    " \
        --log ${PUB_USAGE_DIR}/${PAYLOAD}.txt \
        --include-children \
        --duration $TEST_TIME

    killall -9 $SUB_PROGRAM
    sleep 1

done
