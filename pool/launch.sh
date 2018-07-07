#!/bin/bash
rm -rf ./data && mkdir -p ./data
./simple-mining-pool generate-config ./config.toml  data/pub.toml data/sec.toml --peer-address 127.0.0.1:6331
./simple-mining-pool finalize --public-api-address 0.0.0.0:9200 --private-api-address 0.0.0.0:9091 data/sec.toml data/node_cfg.toml --public-configs data/pub.toml
./simple-mining-pool run --node-config data/node_cfg.toml --db-path data/db --public-api-address 0.0.0.0:9200
