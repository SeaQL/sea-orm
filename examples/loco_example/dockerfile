FROM rust:1.74-slim as builder

WORKDIR /usr/src/

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /usr/app

COPY --from=builder /usr/src/frontend/dist /usr/app/frontend/dist
COPY --from=builder /usr/src/frontend/dist/index.html /usr/app/frontend/dist/index.html
COPY --from=builder /usr/src/config /usr/app/config
COPY --from=builder /usr/src/target/release/todolist-cli /usr/app/todolist-cli

ENTRYPOINT ["/usr/app/todolist-cli"]