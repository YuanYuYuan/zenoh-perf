SHELL := /bin/bash

build: build_router build_throughput build_latency

build_throughput:
	@cargo build --release \
		--features="shared-memory" \
		--bin zenoh_pub_thr \
		--bin zenoh_sub_thr \
		--bin zenoh_pub_shm

build_latency:
	@cargo build --release \
		--bin zenoh_ping \
		--bin zenoh_pong
build_router:
	@git -C zenoh pull &> /dev/null || git clone https://github.com/eclipse-zenoh/zenoh
	@cd zenoh && cargo build --release --bin zenohd

clean:
	@cargo clean
	@rm Cargo.lock
	@cd zenoh && cargo clean
