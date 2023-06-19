FROM rust:1-bookworm as builder
WORKDIR /usr/src/pokerole-discord-bot
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim
RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/pokerole-discord-bot /usr/local/bin/pokerole-discord-bot
CMD ["pokerole-discord-bot"]
