FROM rust:1.82.0-bookworm as builder
WORKDIR /usr/src/pokerole-discord-bot
COPY . .

ARG DATABASE_URL="sqlite:///build-db.sqlite"

RUN cargo install sqlx-cli
RUN sqlx database create
RUN sqlx migrate run
RUN cargo sqlx prepare
RUN cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/pokerole-discord-bot /usr/local/bin/pokerole-discord-bot
CMD ["pokerole-discord-bot"]
