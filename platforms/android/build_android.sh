#!/bin/bash
# filepath: h:\PROJECTS\webassembly_unified_frontbackend\platforms\android\build_android.sh

# =============================================================================
#                     WEBASSEMBLY UNIFIED ANDROID BUILD SCRIPT
# =============================================================================
# This script builds an Android APK for the WebAssembly Unified project.
# Usage: ./build_android.sh [--clean]
# =============================================================================

echo ""
echo "==============================================================================="
echo "                     WEBASSEMBLY UNIFIED - ANDROID BUILD"
echo "==============================================================================="
echo ""

# Check if clean option is requested
if [ "$1" = "--clean" ]; then
    ./clean_android
    exit 0
fi

echo "[STEP 1/8] Checking prerequisites..."
echo "-----------------------------------------------------------------------"

# Check Rust installation
if ! command -v rustc &> /dev/null; then
    echo "[ERROR] Rust is not installed. Please install Rust first."
    echo "        Visit: https://rustup.rs/"
    read -p "Press enter to continue..."
    exit 1
fi
echo "[OK] Rust found:"
rustc --version

echo ""
echo "[STEP 2/8] Setting up Java JDK 17..."
echo "-----------------------------------------------------------------------"

# Force the use of local Java JDK 17 (for Gradle compatibility)
LOCAL_JDK="$(pwd)/jdk17"

if [ ! -d "$LOCAL_JDK/jdk-17.0.2" ]; then
    echo "[INFO] Installing Java JDK 17 locally..."
    
    echo "    * Downloading Java JDK 17..."
    if command -v wget &> /dev/null; then
        wget -O jdk17.tar.gz "https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_linux-x64_bin.tar.gz" 2>/dev/null
    elif command -v curl &> /dev/null; then
        curl -L -o jdk17.tar.gz "https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_linux-x64_bin.tar.gz"
    else
        echo "[ERROR] Neither wget nor curl found. Cannot download JDK."
        exit 1
    fi
    
    if [ $? -ne 0 ]; then
        echo "[ERROR] Failed to download Java JDK 17"
        read -p "Press enter to continue..."
        exit 1
    fi
    
    # Extract JDK locally
    mkdir -p "$LOCAL_JDK"
    echo "    * Extracting JDK..."
    tar -xzf jdk17.tar.gz -C "$LOCAL_JDK"
    
    if [ $? -ne 0 ]; then
        echo "[ERROR] Failed to extract Java JDK 17"
        read -p "Press enter to continue..."
        exit 1
    fi
    
    rm jdk17.tar.gz
    echo "[OK] Java JDK 17 installed locally"
else
    echo "[OK] Local Java JDK 17 found"
fi

# Configure JAVA_HOME to use local version
export JAVA_HOME="$(find "$LOCAL_JDK" -name "jdk-*" -type d | head -1)"
export PATH="$JAVA_HOME/bin:$PATH"
echo "[INFO] Using local Java: $JAVA_HOME"

# Verify that Java 17 is being used
if ! java -version 2>&1 | grep -q "17\."; then
    echo "[ERROR] Java 17 is not properly configured"
    java -version
    read -p "Press enter to continue..."
    exit 1
fi
echo "[OK] Java 17 properly configured"

echo ""
echo "[STEP 3/8] Setting up cargo-ndk..."
echo "-----------------------------------------------------------------------"

# Install cargo-ndk locally
LOCAL_CARGO_HOME="$(pwd)/local-cargo"
export CARGO_HOME="$LOCAL_CARGO_HOME"
export PATH="$LOCAL_CARGO_HOME/bin:$PATH"

if [ ! -f "$LOCAL_CARGO_HOME/bin/cargo-ndk" ]; then
    echo "[INFO] Installing cargo-ndk locally..."
    mkdir -p "$LOCAL_CARGO_HOME"
    cargo install --root "$LOCAL_CARGO_HOME" cargo-ndk
    if [ $? -ne 0 ]; then
        echo "[ERROR] Failed to install cargo-ndk"
        read -p "Press enter to continue..."
        exit 1
    fi
    echo "[OK] cargo-ndk installed locally"
else
    echo "[OK] cargo-ndk found locally"
fi

echo ""
echo "[STEP 4/8] Setting up Android SDK..."
echo "-----------------------------------------------------------------------"

# Configure local Android SDK
LOCAL_ANDROID_SDK="$(pwd)/android-sdk"
export ANDROID_HOME="$LOCAL_ANDROID_SDK"
export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"

echo "[INFO] Android SDK location: $LOCAL_ANDROID_SDK"

# Install Android SDK if necessary
if [ ! -d "$LOCAL_ANDROID_SDK/cmdline-tools" ]; then
    echo "[INFO] Installing Android SDK..."
    
    # Create SDK folder
    mkdir -p "$LOCAL_ANDROID_SDK"
    cd "$LOCAL_ANDROID_SDK"
    
    # Download command line tools
    echo "    * Downloading Android Command Line Tools..."
    if command -v wget &> /dev/null; then
        wget -O cmdline-tools.zip "https://dl.google.com/android/repository/commandlinetools-linux-9477386_latest.zip"
    else
        curl -L -o cmdline-tools.zip "https://dl.google.com/android/repository/commandlinetools-linux-9477386_latest.zip"
    fi
    
    if [ $? -ne 0 ]; then
        echo "[ERROR] Failed to download Android Command Line Tools"
        read -p "Press enter to continue..."
        exit 1
    fi
    
    # Extract tools
    echo "    * Extracting tools..."
    unzip -q cmdline-tools.zip
    
    # Reorganize folder structure
    mkdir -p cmdline-tools/latest
    mv cmdline-tools/bin cmdline-tools/latest/
    mv cmdline-tools/lib cmdline-tools/latest/
    mv cmdline-tools/NOTICE.txt cmdline-tools/latest/
    mv cmdline-tools/source.properties cmdline-tools/latest/
    
    # Clean up
    rm cmdline-tools.zip
    
    echo "[OK] Android Command Line Tools installed"
    cd - > /dev/null
else
    echo "[OK] Android SDK found"
fi

# Update PATH for this session
export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"

# Install necessary SDK components
echo "[INFO] Checking Android SDK components..."
if [ ! -d "$ANDROID_HOME/platform-tools" ]; then
    echo "[INFO] Accepting licenses and installing SDK components..."
    
    # Accept all licenses
    yes | "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --licenses
    
    echo "[INFO] Installing necessary SDK components..."
    "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" "platform-tools" "platforms;android-34" "build-tools;34.0.0" "ndk;25.2.9519653"
    
    if [ $? -ne 0 ]; then
        echo "[ERROR] Failed to install SDK components"
        read -p "Press enter to continue..."
        exit 1
    fi
    echo "[OK] SDK components installed"
else
    echo "[OK] SDK components found"
fi

# Configure NDK
export ANDROID_NDK_ROOT="$ANDROID_HOME/ndk/25.2.9519653"
export NDK_HOME="$ANDROID_NDK_ROOT"
echo "[INFO] NDK location: $ANDROID_NDK_ROOT"

echo ""
echo "[STEP 5/8] Adding Android targets to Rust..."
echo "-----------------------------------------------------------------------"

# Check and add targets only if they are not present
echo "[INFO] Adding Android targets:"
echo "    * aarch64-linux-android (ARM64)"
rustup target add aarch64-linux-android
echo "    * armv7-linux-androideabi (ARM32)"
rustup target add armv7-linux-androideabi
echo "    * x86_64-linux-android (x64)"
rustup target add x86_64-linux-android
echo "    * i686-linux-android (x86)"
rustup target add i686-linux-android

echo ""
echo "[STEP 6/8] Compiling Rust code for Android..."
echo "-----------------------------------------------------------------------"

# Build for all Android architectures
CARGO_NDK_PATH="$LOCAL_CARGO_HOME/bin/cargo-ndk"
if [ ! -f "$CARGO_NDK_PATH" ]; then
    echo "[ERROR] cargo-ndk not found at $CARGO_NDK_PATH"
    read -p "Press enter to continue..."
    exit 1
fi

echo "[INFO] Building for multiple architectures..."

echo "    * Building for aarch64-linux-android (ARM64)..."
cargo ndk -t aarch64-linux-android -p 21 -- build --release
if [ $? -ne 0 ]; then
    echo "[ERROR] Failed to compile for aarch64-linux-android"
    read -p "Press enter to continue..."
    exit 1
fi

echo "    * Building for armv7-linux-androideabi (ARM32)..."
cargo ndk -t armv7-linux-androideabi -p 21 -- build --release
if [ $? -ne 0 ]; then
    echo "[ERROR] Failed to compile for armv7-linux-androideabi"
    read -p "Press enter to continue..."
    exit 1
fi

echo "    * Building for x86_64-linux-android (x64)..."
cargo ndk -t x86_64-linux-android -p 21 -- build --release
if [ $? -ne 0 ]; then
    echo "[ERROR] Failed to compile for x86_64-linux-android"
    read -p "Press enter to continue..."
    exit 1
fi

echo "    * Building for i686-linux-android (x86)..."
cargo ndk -t i686-linux-android -p 21 -- build --release
if [ $? -ne 0 ]; then
    echo "[ERROR] Failed to compile for i686-linux-android"
    read -p "Press enter to continue..."
    exit 1
fi

echo "[OK] All architectures compiled successfully"

echo ""
echo "[STEP 7/8] Copying native libraries and static files..."
echo "-----------------------------------------------------------------------"

# Copy compiled libraries
echo "[INFO] Copying native libraries to JNI folders..."
JNI_LIBS="app/src/main/jniLibs"

mkdir -p "$JNI_LIBS/arm64-v8a"
mkdir -p "$JNI_LIBS/armeabi-v7a"
mkdir -p "$JNI_LIBS/x86_64"
mkdir -p "$JNI_LIBS/x86"

echo "    * Copying ARM64 library..."
cp "../../target/aarch64-linux-android/release/libwebassembly_android.so" "$JNI_LIBS/arm64-v8a/"
echo "    * Copying ARM32 library..."
cp "../../target/armv7-linux-androideabi/release/libwebassembly_android.so" "$JNI_LIBS/armeabi-v7a/"
echo "    * Copying x64 library..."
cp "../../target/x86_64-linux-android/release/libwebassembly_android.so" "$JNI_LIBS/x86_64/"
echo "    * Copying x86 library..."
cp "../../target/i686-linux-android/release/libwebassembly_android.so" "$JNI_LIBS/x86/"

if [ $? -ne 0 ]; then
    echo "[ERROR] Failed to copy native libraries"
    read -p "Press enter to continue..."
    exit 1
fi
echo "[OK] Native libraries copied successfully"

# Install Gradle Wrapper JAR if necessary
echo "[INFO] Checking Gradle Wrapper..."
if [ ! -f "gradle/wrapper/gradle-wrapper.jar" ]; then
    echo "[INFO] Downloading Gradle Wrapper..."
    mkdir -p "gradle/wrapper"
    
    if command -v wget &> /dev/null; then
        wget -O gradle-8.5-bin.zip "https://services.gradle.org/distributions/gradle-8.5-bin.zip"
    else
        curl -L -o gradle-8.5-bin.zip "https://services.gradle.org/distributions/gradle-8.5-bin.zip"
    fi
    
    unzip -q gradle-8.5-bin.zip
    cp "gradle-8.5/lib/gradle-wrapper.jar" "gradle/wrapper/"
    rm -rf gradle-8.5 gradle-8.5-bin.zip
    echo "[OK] Gradle Wrapper installed locally"
else
    echo "[OK] Gradle Wrapper found"
fi

echo ""
echo "[INFO] Copying client static files..."
echo "    ---------------------------------------------------------------"

# Variables for static files
SOURCE_STATIC_DIR="../../client/static"
ASSETS_STATIC_DIR="app/src/main/assets/static"
ASSETS_PKG_DIR="app/src/main/assets/static/pkg"

echo "[INFO] Source: $SOURCE_STATIC_DIR"
echo "[INFO] Destination: $ASSETS_STATIC_DIR"

# Create assets directories if they don't exist
mkdir -p "$ASSETS_STATIC_DIR"
mkdir -p "$ASSETS_PKG_DIR"

# Remove old files
echo "[INFO] Cleaning old files..."
rm -rf "$ASSETS_STATIC_DIR"/*
mkdir -p "$ASSETS_STATIC_DIR"
mkdir -p "$ASSETS_PKG_DIR"

# Copy all static files
echo "[INFO] Copying static files..."
cp -r "$SOURCE_STATIC_DIR"/* "$ASSETS_STATIC_DIR/"

if [ $? -ne 0 ]; then
    echo "[ERROR] Failed to copy static files"
    read -p "Press enter to continue..."
    exit 1
fi

echo "[OK] Static files copied successfully"
echo "[INFO] Files copied to assets/static:"
ls -1 "$ASSETS_STATIC_DIR"

echo ""
echo "[STEP 8/8] Building Android APK..."
echo "-----------------------------------------------------------------------"
echo "[INFO] Building Android APK..."
./gradlew assembleDebug

if [ $? -ne 0 ]; then
    echo "[ERROR] Failed to build APK"
    read -p "Press enter to continue..."
    exit 1
fi

echo ""
echo "[OK] APK built successfully!"
echo "[INFO] File location: app/build/outputs/apk/debug/app-debug.apk"
echo "==============================================================================="

echo ""
echo "[INFO] Installing on connected device..."
echo "-----------------------------------------------------------------------"
"$ANDROID_HOME/platform-tools/adb" install -r "app/build/outputs/apk/debug/app-debug.apk"

if [ $? -ne 0 ]; then
    echo "[WARNING] Installation failed or no device connected"
else
    echo "[OK] APK installed successfully"
fi

echo ""
echo "[INFO] Starting application..."
adb shell am start -n com.webassembly.unified/.MainActivity

echo ""
echo "[INFO] Connected device(s):"
adb devices

echo ""
echo "[INFO] Monitoring logs (Ctrl+C to stop)..."
echo "-----------------------------------------------------------------------"
adb logcat -c
adb logcat -s WebAssemblyApp