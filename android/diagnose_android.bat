@echo off
setlocal enabledelayedexpansion

echo 🔍 Diagnostic de l'environnement Android pour Rust
echo ================================================
echo.

REM Vérification de Rust
echo 📦 Vérification de Rust...
rustc --version >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo ❌ Rust n'est pas installé ou pas dans le PATH
    echo    Installez Rust depuis https://rustup.rs/
) else (
    echo ✅ Rust installé :
    rustc --version
    cargo --version
)
echo.

REM Vérification des cibles Android
echo 🎯 Vérification des cibles Android...
for %%t in (aarch64-linux-android armv7-linux-androideabi x86_64-linux-android) do (
    rustup target list | findstr /C:"%%t (installed)" >nul
    if !ERRORLEVEL! equ 0 (
        echo ✅ Cible %%t installée
    ) else (
        echo ❌ Cible %%t manquante - Exécutez: rustup target add %%t
    )
)
echo.

REM Recherche du Android SDK
echo 📂 Recherche du Android SDK...
set SDK_FOUND=0

if exist "%ANDROID_HOME%" (
    echo ✅ ANDROID_HOME défini : %ANDROID_HOME%
    set SDK_PATH=%ANDROID_HOME%
    set SDK_FOUND=1
) else (
    echo ⚠️  Variable ANDROID_HOME non définie
)

if exist "%USERPROFILE%\AppData\Local\Android\Sdk" (
    echo ✅ SDK trouvé : %USERPROFILE%\AppData\Local\Android\Sdk
    if !SDK_FOUND! equ 0 (
        set SDK_PATH=%USERPROFILE%\AppData\Local\Android\Sdk
        set SDK_FOUND=1
    )
) else (
    echo ❌ SDK non trouvé : %USERPROFILE%\AppData\Local\Android\Sdk
)

if exist "C:\Android\Sdk" (
    echo ✅ SDK trouvé : C:\Android\Sdk
    if !SDK_FOUND! equ 0 (
        set SDK_PATH=C:\Android\Sdk
        set SDK_FOUND=1
    )
) else (
    echo ❌ SDK non trouvé : C:\Android\Sdk
)

if !SDK_FOUND! equ 0 (
    echo.
    echo ❌ Aucun Android SDK trouvé !
    echo    Installez Android Studio depuis https://developer.android.com/studio
    goto :end
)
echo.

REM Recherche du NDK
echo 🔧 Recherche du Android NDK...
if exist "%SDK_PATH%\ndk" (
    echo ✅ Dossier NDK trouvé : %SDK_PATH%\ndk
    echo 📦 Versions NDK disponibles :
    for /f %%i in ('dir "%SDK_PATH%\ndk" /b /ad 2^>nul') do (
        echo    - %%i
        set LATEST_NDK=%%i
    )
    
    if defined LATEST_NDK (
        echo.
        echo 🔍 Vérification du NDK %LATEST_NDK%...
        set NDK_PATH=%SDK_PATH%\ndk\!LATEST_NDK!
        
        if exist "!NDK_PATH!\toolchains\llvm\prebuilt\windows-x86_64\bin\llvm-ar.exe" (
            echo ✅ Outils de compilation trouvés
        ) else (
            echo ❌ Outils de compilation manquants dans !NDK_PATH!
        )
    )
) else (
    echo ❌ Aucun NDK trouvé dans %SDK_PATH%\ndk
    echo.
    echo 💡 Pour installer le NDK :
    echo    1. Ouvrez Android Studio
    echo    2. Tools ^> SDK Manager
    echo    3. Onglet SDK Tools
    echo    4. Cochez "NDK (Side by side)"
    echo    5. Cliquez Apply
)
echo.

REM Vérification du projet Rust
echo 🦀 Vérification du projet Rust...
if exist "Cargo.toml" (
    echo ✅ Cargo.toml trouvé
    
    findstr /C:"[[bin]]" Cargo.toml >nul
    if !ERRORLEVEL! equ 0 (
        echo ✅ Configuration binaire trouvée
    ) else (
        echo ⚠️  Aucune configuration [[bin]] trouvée dans Cargo.toml
    )
    
    findstr /C:"crate-type" Cargo.toml >nul
    if !ERRORLEVEL! equ 0 (
        echo ✅ Configuration de bibliothèque trouvée
    ) else (
        echo ⚠️  Aucune configuration crate-type trouvée (optionnel pour Android)
    )
) else (
    echo ❌ Cargo.toml non trouvé - Assurez-vous d'être à la racine du projet
)
echo.

echo 📋 Résumé du diagnostic :
if !SDK_FOUND! equ 1 (
    echo ✅ Android SDK : Trouvé
) else (
    echo ❌ Android SDK : Manquant
)

if defined LATEST_NDK (
    echo ✅ Android NDK : Version !LATEST_NDK!
) else (
    echo ❌ Android NDK : Manquant
)

echo.
echo 🚀 Si tout est vert, vous pouvez exécuter build_rust_android.bat
echo.

:end
pause
