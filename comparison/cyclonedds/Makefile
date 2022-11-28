all: install build

install:
	@git clone https://github.com/eclipse-cyclonedds/cyclonedds
	@git -C cyclonedds apply ../mod-for-perf.patch

build:
	@cmake -S cyclonedds -B target
	@cmake --build target -j
