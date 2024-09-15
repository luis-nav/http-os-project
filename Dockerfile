FROM rust:latest

WORKDIR /usr/src/app

COPY . /usr/src/app

RUN cargo build --release

CMD cargo run