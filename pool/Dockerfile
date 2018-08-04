# Exonum build envinroment
FROM rust:1.27.2-stretch as build

ENV ROCKSDB_LIB_DIR=/usr/lib/x86_64-linux-gnu
ENV SNAPPY_LIB_DIR=/usr/lib/x86_64-linux-gnu

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        libsodium-dev \
        libsnappy-dev \
        librocksdb-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Creates a dummy project used to grab dependencies
RUN USER=root cargo new --bin simple-mining-pool
WORKDIR /simple-mining-pool

# Copies over *only* your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Builds your dependencies and removes the
# fake source code from the dummy project
RUN cargo build
RUN rm src/*.rs

# Copies only your actual source code to
# avoid invalidating the cache at all
COPY ./src ./src
RUN cargo build
RUN cargo test


# Create a new stage with a minimal image
# because we already have a binary built
FROM rust:1.27.2-slim-jessie

ENV LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/usr/lib

# Host parameters
ENV HOST_IP=127.0.0.1
ENV HOST_PUBLIC_API_PORT=8200
ENV HOST_PRIVATE_API_PORT=8091

# Launch parameters
ENV SIMPLE_CONFIG_PATH=config.toml
ENV SIMPLE_DB_PATH=data/db

# Relative path to binary and command
ENV SIMPLE_BIN_PATH=simple-mining-pool

EXPOSE 8200
EXPOSE 8091

# copies the binary from the "build" stage to the current stage
WORKDIR /run
COPY --from=build /simple-mining-pool/target/debug/simple-mining-pool /usr/bin

# copies libraries
RUN mkdir -p ./lib
COPY --from=build /usr/lib/librocksdb.so.4.5 /usr/lib
COPY --from=build /usr/lib/x86_64-linux-gnu/libsnappy.so.1 /usr/lib
COPY --from=build /usr/lib/x86_64-linux-gnu/libgflags.so.2 /usr/lib
COPY --from=build /usr/lib/x86_64-linux-gnu/libstdc++.so.6 /usr/lib
COPY --from=build /usr/lib/x86_64-linux-gnu/libssl.so /usr/lib/libssl.so.1.1
COPY --from=build /usr/lib/x86_64-linux-gnu/libcrypto.so /usr/lib/libcrypto.so.1.1

# configure node and set default behaviour (TO DO parametrize via entrypoint)
RUN mkdir -p ./data/db
VOLUME /run/data

COPY config.toml ./
COPY launch.sh ./
CMD ["./launch.sh"]
