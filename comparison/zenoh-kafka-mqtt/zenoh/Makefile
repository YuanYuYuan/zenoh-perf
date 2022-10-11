build: build_router build_throughput build_latency

build_throughput:
	@cargo build --release \
		--bin zenoh_pub_thr \
		--bin zenoh_sub_thr

build_latency:
	@cargo build --release \
		--bin zenoh_ping \
		--bin zenoh_pong

build_router:
	@git -C zenoh pull &> /dev/null || git clone https://github.com/eclipse-zenoh/zenoh
	@cargo build --release --bin zenohd

build_examples:
	@cd zenoh && cargo build --release \
		--example z_pub_thr \
		--example z_sub_thr

clean:
	@cargo clean
