FROM rust:1.94-slim AS BUILDER

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim as RUNTIME
RUN apt-get update 
WORKDIR /app 
COPY --from=builder /app/target/release/simple-email-server .

EXPOSE 2525 8080

CMD [ "./simple-email-server" ]