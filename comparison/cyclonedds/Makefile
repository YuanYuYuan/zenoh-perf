all: install build_cyclonedds build_throughput build_latency

install:
	@git -C cyclonedds pull || git clone https://github.com/eclipse-cyclonedds/cyclonedds

build_cyclonedds:
	@cmake -S cyclonedds -B build -D CMAKE_INSTALL_PREFIX=install -D CMAKE_BUILD_TYPE=Release
	@cmake --build build --target install --config Release -j

build_throughput:
	@cmake -S throughput -B throughput/target -D CMAKE_PREFIX_PATH=../install -D CMAKE_BUILD_TYPE=Release
	@cmake --build throughput/target --config Release -j

build_latency:
	@cmake -S latency -B latency/target -D CMAKE_PREFIX_PATH=../install -D CMAKE_BUILD_TYPE=Release
	@cmake --build latency/target --config Release -j

clean:
	@rm -r build install throughput/target latency/target
