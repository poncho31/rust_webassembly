#!/bin/bash

# Script pour compiler le serveur Rust pour Android
echo "🔧 Configuration de l'environnement de compilation Android..."

# Installation des cibles Android si pas déjà installées
echo "📦 Installation des cibles Android..."
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android
rustup target add i686-linux-android

# Configuration du NDK Android (à adapter selon votre installation)
export ANDROID_NDK_ROOT="$HOME/Android/Sdk/ndk/25.2.9519653"  # Modifiez selon votre version NDK

if [ ! -d "$ANDROID_NDK_ROOT" ]; then
    echo "❌ Android NDK non trouvé à $ANDROID_NDK_ROOT"
    echo "   Veuillez installer Android NDK et mettre à jour la variable ANDROID_NDK_ROOT"
    exit 1
fi

# Configuration des compilateurs croisés
export AR_aarch64_linux_android="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
export CC_aarch64_linux_android="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang"
export CXX_aarch64_linux_android="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang++"

export AR_armv7_linux_androideabi="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
export CC_armv7_linux_androideabi="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi21-clang"
export CXX_armv7_linux_androideabi="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi21-clang++"

export AR_x86_64_linux_android="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
export CC_x86_64_linux_android="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android21-clang"
export CXX_x86_64_linux_android="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android21-clang++"

echo "🏗️  Compilation pour Android..."

# Compilation pour ARM64 (architecture principale des téléphones modernes)
echo "📱 Compilation pour ARM64 (aarch64)..."
cargo build --release --target aarch64-linux-android --bin server

# Compilation pour ARM 32-bit (anciens téléphones)
echo "📱 Compilation pour ARM 32-bit (armv7)..."
cargo build --release --target armv7-linux-androideabi --bin server

# Compilation pour x86_64 (émulateur Android)
echo "🖥️  Compilation pour x86_64 (émulateur)..."
cargo build --release --target x86_64-linux-android --bin server

echo "✅ Compilation terminée !"
echo ""
echo "📋 Fichiers générés :"
echo "  - target/aarch64-linux-android/release/server"
echo "  - target/armv7-linux-androideabi/release/server"  
echo "  - target/x86_64-linux-android/release/server"
echo ""
echo "🚀 Vous pouvez maintenant compiler l'application Android avec:"
echo "   cd android && ./gradlew assembleDebug"
