FROM rust:1.85-slim AS builder

WORKDIR /app
COPY Cargo.toml .
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim

RUN useradd -m -s /bin/bash aish
COPY --from=builder /app/target/release/aish /usr/local/bin/aish

USER aish
WORKDIR /home/aish

ENV AISH_RUNTIME=mock

ENTRYPOINT ["aish"]
