FROM rust:slim-buster as builder

ADD . ./
RUN apt update
RUN apt install -y pkg-config libssl-dev libsqlite3-dev
RUN rustup update nightly
RUN cargo +nightly build --release

FROM debian:buster-slim
RUN apt update
RUN apt install -y openssl libsqlite3-0 ca-certificates
RUN update-ca-certificates
COPY --from=builder ./target/release/quiz-bot /quiz-bot
CMD ["/quiz-bot"]