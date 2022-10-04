This MQTT performance test code is the modified verision of: https://github.com/ZettaScaleLabs/zenoh-perf/tree/master/comparison/mqtt

## Compiling
To compile the test code, Paho MQTT C Client need to be installed first:
```bash=
apt-get install openssl libssl-dev
cd /usr/local/src
git clone https://github.com/eclipse/paho.mqtt.c.git
cd paho.mqtt.c.git
make && sudo make install
```

Compile the code with:
```bash
make -f Makefile
```

The executables can be found in "./target" folder.

## Default settings
### Default settings of MQTT broker
The setting of MQTT broker can be modified in ```./mosquitto.conf```
Current default setting includes:
* Listen on 1883 port
* max\_inflight\_messages = 0 (unlimited)
* max\_queued\_messages = 0 (unlimited)

### Default settings of throughput test
Hard coded default settings:
* QoS level of clients are set to 1 (at least once)
* Default topic: "/test/thr"
* Default client ID: "mqtt\_pub\_thr", "mqtt\_sub\_thr"

Default settings that can be modified with options:
* Default broker: tcp:/127.0.0.1:1883

### Default settings of latency test
Hard coded default settings:
* QoS level of clients are set to 0 (best effort)
* Default topic: "/test/ping", "/test/pong"
* Default client ID: "mqtt\_ping", "mqtt\_pong"

Default settings that can be modified with options:
* Default broker: tcp:/127.0.0.1:1883
* Default test interval: 1s



## Running MQTT throughput test
The common options for publisher and subscribers are:
* -b : MQTT broker locator, e.g. tcp:/127.0.0.1:1883
* -p : MQTT message payload size in bytes, max value equal to 268435400

Publiser options:
* -t : Printing the throughput measured in publisher side or not, 0(not print) or 1(print)

Example:
```bash=
# Run MQTT broker on local machine or docker container
docker run --init -it -p 1235:1883 --rm -v <absolute path>/mqtt/mosquitto.conf:/mosquitto/config/mosquitto.conf eclipse-mosquitto

# Run subscriber
./mqtt_sub_thr -p 1024 -b tcp://127.0.0.1:1235

Run publisher
./mqtt_pub_thr -p 1024 -t 1 -b tcp://127.0.0.1:1235
```

Output format:
```<payload>,<msg/s>```

## Running MQTT latency test
The common options for publisher and subscribers are:
* -b : MQTT broker locator, e.g. tcp:/127.0.0.1:1883

Publiser options:
* -i : Time interval between two latency test
* -p : MQTT message payload size in bytes, max value equal to 268435400

Example:
```bash=
# Run MQTT broker on local machine or docker container
docker run --init -it -p 1235:1883 --rm -v <absolute path>/mqtt/mosquitto.conf:/mosquitto/config/mosquitto.conf eclipse-mosquitto:1.6.9

# Run subscriber
./mqtt_pong -b tcp://127.0.0.1:1235

Run publisher
./mqtt_ping -p 8 -b tcp://127.0.0.1:1235
```

Output format:
```<time interval s>,<latency us>```
