# syntax=docker/dockerfile:1
# Backend Rust — API seule (ne sert plus le frontend).

FROM rust:1-slim AS build
WORKDIR /be
COPY . ./
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
COPY --from=build /be/target/release/optimiseur-coupe /app/optimiseur-coupe
ENV PORT=8080
ENV RUST_LOG=info
EXPOSE 8080
CMD ["/app/optimiseur-coupe"]
