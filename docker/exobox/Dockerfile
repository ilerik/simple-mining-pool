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