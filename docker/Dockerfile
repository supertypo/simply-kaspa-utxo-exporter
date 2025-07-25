##
# builder image
##
FROM rust:1.83-alpine AS builder

ARG REPO_DIR=.

RUN apk --no-cache add \
  musl-dev \
  g++ \
  clang15-dev \
  linux-headers \
  openssl-dev \
  postgresql-dev \
  curl

WORKDIR /work

ENV RUSTFLAGS="-C target-feature=-crt-static -C link-args=-s" \
  CARGO_REGISTRIES_CRATES_IO_PROTOCOL="sparse"

# Cache dependencies
COPY "$REPO_DIR"/Cargo.toml "$REPO_DIR"/Cargo.lock ./
COPY "$REPO_DIR"/cli/Cargo.toml ./cli/
COPY "$REPO_DIR"/database/Cargo.toml ./database/
COPY "$REPO_DIR"/exporter/Cargo.toml ./exporter/
RUN mkdir cli/src database/src exporter/src && \
    echo "fn main() {}" > cli/build.rs && \
    echo "pub fn placeholder() {}" > cli/src/lib.rs && \
    echo "pub fn placeholder() {}" > database/src/lib.rs && \
    echo "pub fn placeholder() {}" > exporter/src/lib.rs && \
    echo "fn main() {}" > exporter/src/main.rs

RUN cargo fetch
# Patch missing include in librocksdb-sys-0.16.0+8.10.0
RUN sed -i '1i #include <cstdint>' $(find /usr/local/cargo/registry/src/ -path "*/librocksdb-sys-0.16.0+8.10.0/*/offpeak_time_info.h")
RUN cargo build --release

COPY "$REPO_DIR" .
RUN find . -type f \( -name build.rs -or -name lib.rs -or -name main.rs \) -exec touch {} +
RUN cargo build --release


##
# runtime image
##
FROM alpine

RUN apk --no-cache add \
  libstdc++ \
  postgresql-client

COPY --from=builder /work/target/release/simply-kaspa-utxo-exporter /usr/local/bin/

USER 13337:13337

ARG VERSION
ENV VERSION=$VERSION

ENTRYPOINT ["simply-kaspa-utxo-exporter"]
CMD ["--help"]
