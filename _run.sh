#!/bin/bash

# Charger les variables du .env dans l'environnement
if [ -f ".env" ]; then
    export $(grep -v '^#' .env | xargs)
fi

# Vérifie si le premier argument est 'docker'
if [ "$1" = "docker" ]; then

    docker-compose down
    docker-compose up -d

    # Ouvrir le navigateur (Linux)
    if command -v xdg-open > /dev/null; then
        xdg-open "$ALLOWED_ORIGIN_DOCKER" &
    elif command -v gnome-open > /dev/null; then
        gnome-open "$ALLOWED_ORIGIN_DOCKER" &
    fi

    docker logs -f "$APP_NAME_DOCKER"

else

    # Vérifie la présence de cargo et installe Rust si manquant
    if ! command -v cargo &> /dev/null; then
        echo "[INFO] cargo non trouvé, installation de Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    # Vérifie la présence de wasm-pack et installe si manquant
    if ! command -v wasm-pack &> /dev/null; then
        echo "[INFO] wasm-pack non trouvé, installation..."
        cargo install wasm-pack
    fi
    
    # compile webassembly
    cd client
    wasm-pack build --target web --out-dir static/pkg

    # compile le projet Rust
    cd ..
    cargo build --release

    # Lancer le navigateur en arrière-plan avec délai
    (sleep 5 && if command -v xdg-open > /dev/null; then
        xdg-open "$ALLOWED_ORIGIN"
    elif command -v gnome-open > /dev/null; then
        gnome-open "$ALLOWED_ORIGIN"
    fi) &

    # Démarrer le serveur
    ./target/release/server
fi
