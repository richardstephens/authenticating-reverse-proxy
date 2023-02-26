# 1. This tells docker to use the Rust official image
FROM rust:1.67.1-slim-bullseye as build

WORKDIR /
RUN USER=root cargo new --bin authenticating-rp
WORKDIR /authenticating-rp
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo tree; cargo build --release

RUN rm -rf /authenticating-rp/src; rm -rf /authenticating-rp/target

COPY ./src ./src
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=build /authenticating-rp/target/release/authenticating-rp .
CMD ["./authenticating-rp"]
