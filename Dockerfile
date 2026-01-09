FROM rust:alpine AS builder

ARG TARGETARCH

RUN apk add --no-cache curl mold \
    && rm -rf /var/cache/apk/*

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | sh

RUN cargo binstall cargo-leptos -y

RUN rustup target add wasm32-unknown-unknown

WORKDIR /app
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    mold -run cargo leptos build --release && \
    cp target/release/bird_password /app/ && \
    cp -r target/site /app/site

FROM scratch

WORKDIR /app

COPY --from=builder /app/bird_password /app/
COPY --from=builder /app/site /app/target/site
COPY --from=builder /app/birds.csv /app/
COPY --from=builder /app/bird_names.txt /app/

ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT="target/site"

EXPOSE 8080

ENTRYPOINT ["/app/bird_password"]