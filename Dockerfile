FROM rust:slim-trixie AS builder

RUN apt-get update && apt-get install -y --no-install-recommends mold wget \
    && rm -rf /var/lib/apt/lists/*

RUN wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN tar -xvf cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN cp cargo-binstall /usr/local/cargo/bin

RUN cargo binstall cargo-leptos -y

RUN rustup target add wasm32-unknown-unknown

WORKDIR /app
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    mold -run cargo leptos build --release && \
    cp target/release/bird_password /app/ && \
    cp -r target/site /app/site

FROM debian:trixie-slim

WORKDIR /app

COPY --from=builder /app/bird_password /app/
COPY --from=builder /app/site /app/target/site
COPY --from=builder /app/birds.csv /app/
COPY --from=builder /app/bird_names.txt /app/

ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="target/site"

EXPOSE 8080

CMD ["./bird_password"]
