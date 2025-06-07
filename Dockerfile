# Build stage pour WebAssembly
FROM rust:1.82 AS wasm-builder
RUN cargo install wasm-pack
WORKDIR /app
COPY client/ ./client/
COPY core/ ./core/
WORKDIR /app/client
RUN wasm-pack build --target web --out-dir pkg

# Build stage pour le serveur Rust
FROM rust:1.82 AS rust-builder
WORKDIR /app
COPY . .
COPY --from=wasm-builder /app/client/pkg ./client/pkg
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copier le binaire
COPY --from=rust-builder /app/target/release/server .

# Copier les fichiers statiques et WebAssembly
COPY --from=rust-builder /app/client/static ./client/static
COPY --from=rust-builder /app/client/pkg ./client/pkg

# Copier les certificats si présents
COPY certs/ ./certs/

# Variables d'environnement par défaut (surchargées par docker-compose)
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8088

EXPOSE 8088

CMD ["./server"]
