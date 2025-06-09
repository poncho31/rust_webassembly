# Build stage pour WebAssembly
FROM rust:1.83-alpine AS wasm-builder
RUN apk add --no-cache musl-dev openssl-dev pkgconfig && cargo install wasm-pack
WORKDIR /app
COPY client/ ./client/
COPY core/ ./core/
WORKDIR /app/client
RUN wasm-pack build --target web --out-dir pkg

# Build stage pour le serveur Rust
FROM rust:1.83-alpine AS rust-builder
RUN apk add --no-cache musl-dev openssl-dev pkgconfig
WORKDIR /app
COPY . .
COPY --from=wasm-builder /app/client/pkg ./client/pkg
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM scratch
COPY --from=rust-builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=rust-builder /app/target/x86_64-unknown-linux-musl/release/server /server
COPY --from=rust-builder /app/client/static /client/static
COPY --from=rust-builder /app/client/pkg /client/pkg
COPY certs/ /certs/

EXPOSE 8088

CMD ["/server"]
