FROM rust:1.84.1 AS builder

RUN cargo install --locked trunk
RUN rustup target add wasm32-unknown-unknown
WORKDIR /
COPY . .
RUN cd frontend && trunk build --release
RUN cargo build --bin backend --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /target/release/backend .
COPY --from=builder /dist ./dist/
RUN apt-get update && apt-get install -y libmariadb3 && rm -rf /var/lib/apt/lists/*
RUN apt list --installed
EXPOSE 9223
CMD ["./backend", "--addr", "::", "--port", "9223"]
