FROM public.ecr.aws/docker/library/rust:1.94-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config ./config

RUN cargo build --release --locked

FROM gcr.io/distroless/cc-debian12:nonroot AS runtime

COPY --from=builder --chown=nonroot:nonroot \
    /app/target/release/domain-replay-supervisor-app \
    /usr/local/bin/domain-replay-supervisor-app
COPY --from=builder --chown=nonroot:nonroot \
    /app/config/domain-replay-manifest.example.json \
    /opt/nangman-crypto/domains/runtime/domain-replay-manifest.dev.json

USER nonroot:nonroot

ENV AWS_SDK_LOAD_CONFIG=1

ENTRYPOINT ["/usr/local/bin/domain-replay-supervisor-app"]
