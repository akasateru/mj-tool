# Render などで mj-api をデプロイする用。DB は Supabase（DATABASE_URL）を想定。
FROM rust:1.93-bookworm AS builder
WORKDIR /app

# 依存だけ先にビルド（キャッシュ用）
COPY Cargo.toml Cargo.lock ./
COPY mj-api/Cargo.toml ./mj-api/
COPY mj-domain/Cargo.toml ./mj-domain/
RUN mkdir -p mj-api/src mj-domain/src && \
    echo "fn main() {}" > mj-api/src/main.rs && \
    echo "pub fn dummy() {}" > mj-domain/src/lib.rs && \
    cargo build --release -p mj-api && \
    rm -rf mj-api/src mj-domain/src

# ソースをコピーして本ビルド
COPY mj-api/src ./mj-api/src
COPY mj-domain/src ./mj-domain/src
RUN touch mj-api/src/main.rs mj-domain/src/lib.rs && \
    cargo build --release -p mj-api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 libpq5 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/mj-api /usr/local/bin/
ENV PORT=8080
EXPOSE 8080
CMD ["/usr/local/bin/mj-api"]
