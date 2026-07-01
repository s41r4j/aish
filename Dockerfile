FROM rust:1.85-slim AS builder

WORKDIR /app
COPY Cargo.toml .
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS llama-builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential ca-certificates cmake git \
    && rm -rf /var/lib/apt/lists/*
ARG LLAMA_CPP_REF=master
RUN git clone --depth 1 --branch "${LLAMA_CPP_REF}" https://github.com/ggml-org/llama.cpp.git /llama.cpp \
    && cmake -S /llama.cpp -B /llama.cpp/build -DBUILD_SHARED_LIBS=ON -DLLAMA_CURL=OFF -DGGML_NATIVE=OFF \
    && cmake --build /llama.cpp/build --config Release --target llama-cli -j2

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends bash ca-certificates curl libgomp1 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -s /bin/bash aish
COPY --from=builder /app/target/release/aish /usr/local/bin/aish
COPY --from=llama-builder /llama.cpp/build/bin/ /usr/local/llama/bin/
COPY scripts/download-model.sh /usr/local/bin/aish-download-model
RUN chmod +x /usr/local/bin/aish-download-model \
    && ln -s /usr/local/llama/bin/llama-cli /usr/local/bin/llama-cli \
    && mkdir -p /home/aish/.aish/models \
    && chown -R aish:aish /home/aish/.aish

ENV AISH_RUNTIME=llama.cpp
ENV AISH_LLAMA_BIN=llama-cli
ENV LD_LIBRARY_PATH=/usr/local/llama/bin

USER aish
WORKDIR /home/aish

VOLUME ["/home/aish/.aish"]

ENTRYPOINT ["aish"]
