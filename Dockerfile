FROM rust:latest AS builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY ./src ./src

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/transcoderexpress /usr/bin/

RUN mkdir /input
RUN mkdir /output

CMD ["transcoderexpress", "-i", "/input", "-o", "/output"]
