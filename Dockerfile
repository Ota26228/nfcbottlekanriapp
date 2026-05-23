# ─── ビルドステージ ───────────────────────────────────────────
FROM rust:latest AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

# ─── 実行ステージ ─────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update \
 && apt-get install -y libsqlite3-0 ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/bottlekanri ./
COPY schema.sql ./

EXPOSE 3000

CMD ["./bottlekanri"]
