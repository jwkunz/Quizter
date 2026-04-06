FROM rust:1-bookworm AS builder

WORKDIR /app

COPY server/Cargo.toml server/Cargo.lock /app/server/

RUN mkdir -p /app/server/src \
    && printf 'fn main() {}\n' > /app/server/src/main.rs \
    && cargo build --release --locked --manifest-path /app/server/Cargo.toml \
    && rm -rf /app/server/src

COPY server /app/server

RUN cargo build --release --locked --manifest-path /app/server/Cargo.toml

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/server/target/release/quizster-server /app/quizster-server
COPY web /app/web
COPY assets /app/assets

RUN mkdir -p /app/data

ENV QUIZSTER_HOST=0.0.0.0
ENV QUIZSTER_PORT=8080
ENV QUIZSTER_PUBLIC_BASE_URL=https://quizster.live
ENV QUIZSTER_OPEN_BROWSER=0
ENV QUIZSTER_SPAWN_TERMINAL=0

EXPOSE 8080

CMD ["./quizster-server"]
