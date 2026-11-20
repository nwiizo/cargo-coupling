# syntax=docker/dockerfile:1

ARG APP_NAME=cargo-coupling

# Build with the current stable Rust image
FROM rust:slim-bookworm AS chef
WORKDIR /app

RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG APP_NAME=cargo-coupling

# 依存関係のビルド（キャッシュ可能）
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

# アプリケーションのビルド
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release --bin ${APP_NAME} && \
    cp ./target/release/${APP_NAME} /bin/server

# 本番ステージ：distroless (最小サイズ、Git非対応)
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime

COPY --from=builder /bin/server /app/cargo-coupling
WORKDIR /workspace
EXPOSE 3000
ENTRYPOINT ["/app/cargo-coupling"]
CMD ["--help"]
