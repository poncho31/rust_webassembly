# Build stage pour WebAssembly
FROM rust:1.83-alpine AS wasm-builder
RUN apk add --no-cache musl-dev openssl-dev pkgconfig && cargo install wasm-pack
WORKDIR /app

# Copy dependency files first (change less frequently)
COPY Cargo.toml Cargo.lock ./
COPY core/ ./core/
COPY server/ ./server/

# Copy client dependencies
COPY client/Cargo.toml ./client/
COPY client/static/ ./client/static/

# Copy client source code (invalidates cache when code changes)
COPY client/src/ ./client/src/

# Generate WASM - this layer rebuilds when client src changes
RUN wasm-pack build --target web --out-dir static/pkg client

# Build stage pour le serveur Rust
FROM rust:1.83-alpine AS rust-builder
RUN apk add --no-cache musl-dev openssl-dev pkgconfig
WORKDIR /app

# Copy dependency files first
COPY Cargo.toml Cargo.lock ./
COPY core/ ./core/

# Copy server source code (invalidates cache when server code changes)
COPY server/ ./server/

# Create a modified Cargo.toml without the client member for server-only build
RUN sed 's/"client",//' Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml

# Copy the compiled WASM from previous stage
COPY --from=wasm-builder /app/client/static/pkg ./client/static/pkg

# Build server - this layer rebuilds when server src changes
RUN cargo build --release --target x86_64-unknown-linux-musl --bin server

# Runtime stage minimal
FROM alpine:latest
RUN apk add --no-cache ca-certificates curl
WORKDIR /app

# Créer les répertoires nécessaires
RUN mkdir -p /app/client/static /app/database/migrations /app/storage/files /app/storage/logs /app/certs

# Copy le binaire compilé du stage rust-builder
COPY --from=rust-builder /app/target/x86_64-unknown-linux-musl/release/server /app/server

# Copy les fichiers WASM du stage wasm-builder
COPY --from=wasm-builder /app/client/static /app/client/static

# Rendre le binaire exécutable
RUN chmod +x /app/server

EXPOSE 8090

CMD ["/app/server"]
