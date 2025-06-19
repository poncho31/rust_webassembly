@echo off
setlocal enabledelayedexpansion

REM Script pour compiler le serveur Rust pour Android sur Windows

echo 🔧 Configuration de l'environnement de compilation Android...

REM Installation des cibles Android si pas déjà installées
echo 📦 Installation des cibles Android...
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android
rustup target add i686-linux-android

REM Détection automatique du NDK Android
echo 🔍 Recherche du Android NDK...

set ANDROID_SDK_ROOT=%USERPROFILE%\AppData\Local\Android\Sdk
set ANDROID_NDK_ROOT=

REM Vérification de différents emplacements possibles
if exist "%ANDROID_HOME%\ndk" (
    set SDK_PATH=%ANDROID_HOME%
) else if exist "%USERPROFILE%\AppData\Local\Android\Sdk\ndk" (
    set SDK_PATH=%USERPROFILE%\AppData\Local\Android\Sdk
) else if exist "C:\Android\Sdk\ndk" (
    set SDK_PATH=C:\Android\Sdk
) else (
    echo ❌ Dossier Android SDK non trouvé !
    echo    Veuillez installer Android Studio et le SDK
    goto :error
)

echo 📂 SDK Android trouvé à : %SDK_PATH%

REM Recherche de la version NDK disponible
for /f %%i in ('dir "%SDK_PATH%\ndk" /b /ad 2^>nul') do (
    set NDK_VERSION=%%i
    set ANDROID_NDK_ROOT=%SDK_PATH%\ndk\%%i
    echo 📦 NDK trouvé : version %%i
    goto :ndk_found
)

echo ❌ Aucune version du NDK Android trouvée dans %SDK_PATH%\ndk
echo.
echo 💡 Pour installer le NDK :
echo    1. Ouvrez Android Studio
echo    2. Allez dans Tools ^> SDK Manager
echo    3. Onglet SDK Tools
echo    4. Cochez "NDK (Side by side)"
echo    5. Cliquez Apply et Install
echo.
goto :error

:ndk_found
echo ✅ Utilisation du NDK : %ANDROID_NDK_ROOT%

REM Configuration des compilateurs croisés
set AR_aarch64_linux_android=%ANDROID_NDK_ROOT%\toolchains\llvm\prebuilt\windows-x86_64\bin\llvm-ar.exe
set CC_aarch64_linux_android=%ANDROID_NDK_ROOT%\toolchains\llvm\prebuilt\windows-x86_64\bin\aarch64-linux-android21-clang.exe
set CXX_aarch64_linux_android=%ANDROID_NDK_ROOT%\toolchains\llvm\prebuilt\windows-x86_64\bin\aarch64-linux-android21-clang++.exe

set AR_armv7_linux_androideabi=%ANDROID_NDK_ROOT%\toolchains\llvm\prebuilt\windows-x86_64\bin\llvm-ar.exe
set CC_armv7_linux_androideabi=%ANDROID_NDK_ROOT%\toolchains\llvm\prebuilt\windows-x86_64\bin\armv7a-linux-androideabi21-clang.exe
set CXX_armv7_linux_androideabi=%ANDROID_NDK_ROOT%\toolchains\llvm\prebuilt\windows-x86_64\bin\armv7a-linux-androideabi21-clang++.exe

set AR_x86_64_linux_android=%ANDROID_NDK_ROOT%\toolchains\llvm\prebuilt\windows-x86_64\bin\llvm-ar.exe
set CC_x86_64_linux_android=%ANDROID_NDK_ROOT%\toolchains\llvm\prebuilt\windows-x86_64\bin\x86_64-linux-android21-clang.exe
set CXX_x86_64_linux_android=%ANDROID_NDK_ROOT%\toolchains\llvm\prebuilt\windows-x86_64\bin\x86_64-linux-android21-clang++.exe

echo 🏗️  Compilation pour Android...

REM Vérification que nous sommes dans le bon répertoire
cd /d "%~dp0\.."
if not exist "Cargo.toml" (
    echo ❌ Fichier Cargo.toml non trouvé !
    echo    Assurez-vous d'être à la racine du projet Rust
    goto :error
)

REM Compilation avec cargo-ndk pour toutes les architectures Android
echo 📱 Compilation avec cargo-ndk pour ARM64, ARM32 et x86_64...
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 build --release --bin server
if %ERRORLEVEL% neq 0 (
    echo ❌ Erreur lors de la compilation Android
    echo � Assurez-vous que cargo-ndk est installé : cargo install cargo-ndk
    goto :error
)

echo ✅ Compilation terminée !
echo.
echo 📋 Fichiers générés :
echo   - target\aarch64-linux-android\release\server.exe
echo   - target\armv7-linux-androideabi\release\server.exe
echo   - target\x86_64-linux-android\release\server.exe
echo.
echo 🚀 Vous pouvez maintenant compiler l'application Android avec:
echo    cd android ^&^& gradlew.bat assembleDebug
echo.
echo ✅ Script terminé avec succès !
goto :end

:error
echo.
echo ❌ Script arrêté à cause d'une erreur !
echo.
pause
exit /b 1

:end
pause
