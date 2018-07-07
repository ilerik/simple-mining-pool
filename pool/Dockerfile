FROM rust:1.27.0-stretch as build

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
RUN cargo update && cargo build
#RUN find / -name "librocksdb*"
#RUN find / -name "libsnappy*"
#RUN find / -name  "libgflags.so.2"

# Copies testkit files and runs tests
COPY ./tests ./tests
RUN cargo test

# Create a new stage with a minimal image
# because we already have a binary built
FROM rust:1.27.0-slim-jessie

ENV LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/usr/lib

# copies the binary from the "build" stage to the current stage
WORKDIR /run
COPY --from=build simple-mining-pool/target/debug/simple-mining-pool .

# copies libraries
RUN mkdir -p ./lib
COPY --from=build /usr/lib/librocksdb.so.4.5 /usr/lib
COPY --from=build /usr/lib/x86_64-linux-gnu/libsnappy.so.1 /usr/lib
COPY --from=build /usr/lib/x86_64-linux-gnu/libgflags.so.2 /usr/lib
COPY --from=build /usr/lib/x86_64-linux-gnu/libstdc++.so.6 /usr/lib

# configure node and set default behaviour (TO DO parametrize via entrypoint)
COPY config.toml ./
COPY launch.sh ./
RUN chmod +x launch.sh
ENTRYPOINT ["./launch.sh"]

