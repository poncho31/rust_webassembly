#!/bin/bash
set -e

# Charger les variables du .env dans l'environnement
if [ -f .env ]; then
  export $(grep -v '^#' .env | xargs)
fi

if [[ "$1" == "docker" ]]; then
  docker-compose down
  docker-compose up -d
  if command -v xdg-open > /dev/null; then
    xdg-open "$ALLOWED_ORIGIN_DOCKER" &
  elif command -v gnome-open > /dev/null; then
    gnome-open "$ALLOWED_ORIGIN_DOCKER" &
  elif command -v open > /dev/null; then
    open "$ALLOWED_ORIGIN_DOCKER" &
  fi
  docker logs -f "$APP_NAME_DOCKER"

elif [[ "$1" == "android" ]]; then
  if [[ "$2" == "docker" ]]; then
    echo "[INFO] Lancement de la construction Android avec Docker..."
    cd platforms/android
    ./build_android_docker.sh
    cd ../..
  else
    echo "[INFO] Construction de l'APK Android..."
    cd platforms/android
    ./build_android.sh
    cd ../..
  fi

elif [[ "$1" == "arduino" ]]; then
  if [[ "$2" == "esp32" ]]; then
    echo "[INFO] Déploiement Arduino ESP32..."
    echo "[WARN] Support ESP32 en cours de développement"
    cd platforms/arduino
    # TODO: Ajouter support ESP32
    cargo run --release -- complete
    cd ../..
  else
    echo "[INFO] Déploiement Arduino ESP8266..."
    cd platforms/arduino
    cargo run --release -- complete
    cd ../..
  fi

else
  # Vérifie la présence de cargo et installe Rust si manquant
  if ! command -v cargo &> /dev/null; then
    echo "[INFO] cargo non trouvé, installation de Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
  fi

  # Vérifie la présence de wasm-pack et installe si manquant
  if ! command -v wasm-pack &> /dev/null; then
    echo "[INFO] wasm-pack non trouvé, installation..."
    cargo install wasm-pack
  fi

  # Vérifie la présence de cargo-ndk et installe si manquant
  if ! command -v cargo-ndk &> /dev/null; then
    echo "[INFO] cargo-ndk non trouvé, installation..."
    cargo install cargo-ndk
  fi

  if [[ "$1" == "force" ]]; then
    pkill rust-analyzer || true
    pkill server || true
    cargo clean
    rm -rf client/static/pkg/*
    echo "[INFO] Projet nettoyé."
  fi

  # Compilation WebAssembly
  cd client
  wasm-pack build --target web --out-dir static/pkg
  cd ..

  # Compilation Rust
  cargo build --release

  # Lancement du navigateur
  if command -v xdg-open > /dev/null; then
    xdg-open "$ALLOWED_ORIGIN" &
  elif command -v gnome-open > /dev/null; then
    gnome-open "$ALLOWED_ORIGIN" &
  elif command -v open > /dev/null; then
    open "$ALLOWED_ORIGIN" &
  fi

  # Lancement du serveur
  ./target/release/server
fi