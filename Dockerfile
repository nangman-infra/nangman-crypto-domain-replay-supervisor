# syntax=docker/dockerfile:1.7

FROM public.ecr.aws/docker/library/rust:1.94-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config/domain-replay-manifest.example.json ./config/domain-replay-manifest.example.json

RUN --mount=type=secret,id=domain_replay_manifest,required=false \
    if [ -f /run/secrets/domain_replay_manifest ]; then \
        cp /run/secrets/domain_replay_manifest ./config/domain-replay-manifest.dev.json; \
    else \
        cp ./config/domain-replay-manifest.example.json ./config/domain-replay-manifest.dev.json; \
    fi

ARG CARGO_BUILD_PROFILE=release
RUN if [ "$CARGO_BUILD_PROFILE" = "release" ]; then \
        cargo build --release --locked; \
    elif [ "$CARGO_BUILD_PROFILE" = "debug" ]; then \
        cargo build --locked; \
    else \
        echo "unsupported CARGO_BUILD_PROFILE=$CARGO_BUILD_PROFILE" >&2; \
        exit 1; \
    fi

FROM gcr.io/distroless/cc-debian12:nonroot AS runtime

ARG CARGO_BUILD_PROFILE=release
COPY --from=builder --chown=nonroot:nonroot \
    /app/target/${CARGO_BUILD_PROFILE}/domain-replay-supervisor-app \
    /usr/local/bin/domain-replay-supervisor-app
COPY --from=builder --chown=nonroot:nonroot \
    /app/config/domain-replay-manifest.dev.json \
    /opt/nangman-crypto/domains/runtime/domain-replay-manifest.dev.json

USER nonroot:nonroot

ENV AWS_SDK_LOAD_CONFIG=1

ENTRYPOINT ["/usr/local/bin/domain-replay-supervisor-app"]
