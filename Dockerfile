# Build stage pour WebAssembly
FROM rust:1.83-alpine AS wasm-builder
RUN apk add --no-cache musl-dev openssl-dev pkgconfig && cargo install wasm-pack
WORKDIR /app
# Copy workspace root and all necessary files for workspace resolution
COPY Cargo.toml ./
COPY Cargo.lock ./
COPY client/ ./client/
COPY core/ ./core/
COPY server/ ./server/
# Run wasm-pack from workspace root, specifying the client package
RUN wasm-pack build --target web --out-dir pkg client

# Build stage pour le serveur Rust
FROM rust:1.83-alpine AS rust-builder
RUN apk add --no-cache musl-dev openssl-dev pkgconfig
WORKDIR /app
# Copy workspace root and necessary files for server build
COPY Cargo.toml ./
COPY Cargo.lock ./
COPY server/ ./server/
COPY core/ ./core/
# Create a modified Cargo.toml without the client member for server-only build
RUN sed 's/"client",//' Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml
# Copy the compiled WASM from previous stage
COPY --from=wasm-builder /app/client/pkg ./client/pkg
RUN cargo build --release --target x86_64-unknown-linux-musl --bin server

# Runtime stage
FROM alpine:latest
RUN apk add --no-cache ca-certificates
WORKDIR /app
COPY --from=rust-builder /app/target/x86_64-unknown-linux-musl/release/server /app/server
COPY --from=wasm-builder /app/client/static /app/client/static
COPY --from=wasm-builder /app/client/pkg /app/client/pkg
COPY certs/ /app/certs/

EXPOSE 8088

CMD ["/app/server"]
