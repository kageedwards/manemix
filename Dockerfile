# Build stage
FROM rust:1.88-bookworm AS builder

WORKDIR /build

# 1) Cache dependencies: copy manifests + lockfile, create stub src, build deps
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/updatetags.rs && \
    echo "fn main() {}" > src/bin/admin.rs && \
    cargo build --release && \
    rm -rf src target/release/manemix target/release/manemix-updatetags target/release/manemix-admin \
           target/release/deps/manemix-*

# 2) Copy real source and build (only recompiles our crate, deps are cached)
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 libssl3 ca-certificates \
    ffmpeg file curl \
    && rm -rf /var/lib/apt/lists/*

# Main server binary
COPY --from=builder /build/target/release/manemix /usr/local/bin/manemix
# Tag updater (called by transcode.sh)
COPY --from=builder /build/target/release/manemix-updatetags /usr/local/bin/manemix-updatetags
# Admin CLI (feature, autofeature, fqueue, dumptracks, stats)
COPY --from=builder /build/target/release/manemix-admin /usr/local/bin/manemix-admin

# Transcode script
COPY tools/transcode.sh /usr/local/bin/transcode.sh
RUN chmod +x /usr/local/bin/transcode.sh

# Templates and static assets
COPY templates /usr/share/manemix/templates
COPY sql /usr/share/manemix/sql

ENV MANEMIX_DIR=/var/lib/manemix
ENV MANEMIX_TEMPLATES=/usr/share/manemix/templates/**/*
ENV MANEMIX_BIND=0.0.0.0:8100

# Create data directories
RUN mkdir -p /var/lib/manemix/tracks \
             /var/lib/manemix/art/medium \
             /var/lib/manemix/art/thumb \
             /var/lib/manemix/tmp

VOLUME ["/var/lib/manemix"]
EXPOSE 8100

CMD ["manemix"]
