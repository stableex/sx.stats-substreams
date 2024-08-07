.PHONY: all
all:
	make build
	make pack
	make graph
	make info

.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

.PHONY: protogen
protogen:
	substreams protogen --exclude-paths sf/substreams,google

.PHONY: pack
pack:
	substreams pack

.PHONY: graph
graph:
	substreams graph

.PHONY: info
info:
	substreams info

.PHONY: run
run:
	substreams run -e eos.substreams.pinax.network:443 prom_out -s 304252678 -t +10000 -o jsonl

.PHONY: gui
gui:
	substreams gui -e eos.substreams.pinax.network:443 prom_out -s 304252678 -t +10000

.PHONY: param
param:
	substreams gui -e eos.substreams.pinax.network:443 prom_out -s 304252678 -t +10000 -p prom_out="to=fee.sx"

.PHONY: sink
sink:
	substreams-sink-prometheus run -e eos.substreams.pinax.network:443 https://github.com/pinax-network/substreams/releases/download/eosmechanics-v0.3.5/eosmechanics-v0.3.5.spkg -s 292103130

