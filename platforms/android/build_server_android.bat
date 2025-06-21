@echo off
setlocal EnableDelayedExpansion

echo ========================================
echo   COMPILATION SERVEUR RUST POUR ANDROID
echo ========================================

REM Variables
set ANDROID_TARGETS=aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
set ASSETS_BIN_DIR=%~dp0app\src\main\assets\bin
set RELEASE_DIR=%~dp0..\..\target

REM Créer le répertoire assets/bin si il n'existe pas
if not exist "%ASSETS_BIN_DIR%" (
    mkdir "%ASSETS_BIN_DIR%"
    echo Répertoire assets/bin créé
)

REM Nettoyer les anciens binaires
echo Nettoyage des anciens binaires...
del /q "%ASSETS_BIN_DIR%\server_*" 2>nul

echo.
echo Vérification de cargo-ndk...

REM Vérifier si cargo-ndk est installé
cargo install --list | findstr "cargo-ndk" >nul
if errorlevel 1 (
    echo Installation de cargo-ndk...
    cargo install cargo-ndk
    if errorlevel 1 (
        echo [ERREUR] Échec de l'installation de cargo-ndk
        pause
        exit /b 1
    )
) else (
    echo cargo-ndk déjà installé
)

echo.
echo Compilation du serveur pour les cibles Android...
echo.

REM Ajouter les cibles Android si nécessaire
for %%t in (%ANDROID_TARGETS%) do (
    echo Vérification de la cible %%t...
    rustup target list --installed | findstr "%%t" >nul
    if errorlevel 1 (
        echo Installation de la cible %%t...
        rustup target add %%t
    ) else (
        echo Cible %%t déjà installée
    )
)

echo.
echo Compilation pour chaque cible avec cargo-ndk...
echo.

REM Définir le chemin du NDK Android
set "ANDROID_NDK_ROOT=%~dp0android-sdk\ndk\25.2.9519653"
if not exist "%ANDROID_NDK_ROOT%" (
    set "ANDROID_NDK_ROOT=%~dp0android-sdk\ndk-bundle"
)
if not exist "%ANDROID_NDK_ROOT%" (
    echo [ERREUR] Android NDK non trouvé. Veuillez l'installer.
    echo Recherche dans : %~dp0android-sdk\ndk\25.2.9519653 ou %~dp0android-sdk\ndk-bundle
    pause
    exit /b 1
echo NDK trouvé : %ANDROID_NDK_ROOT%

REM Compiler pour chaque cible
for %%t in (%ANDROID_TARGETS%) do (
    echo [%%t] Compilation en cours...
    
    REM Définir le nom du binaire de sortie
    if "%%t"=="aarch64-linux-android" (
        set BINARY_NAME=server_aarch64
    )
    if "%%t"=="armv7-linux-androideabi" (
        set BINARY_NAME=server_armv7
    )
    if "%%t"=="i686-linux-android" (
        set BINARY_NAME=server_i686
    )
    if "%%t"=="x86_64-linux-android" (
        set BINARY_NAME=server_x86_64
    )
    
    REM Aller au répertoire du serveur
    cd ..\..\server
    
    REM Compilation avec cargo-ndk
    cargo ndk --target %%t --android-platform 21 -- build --release --bin server
    
    if errorlevel 1 (
        echo [ERREUR] Échec de la compilation pour %%t
        cd ..\platforms\android
        goto next_target
    )
    
    REM Copier le binaire vers assets
    set SOURCE_BINARY=..\target\%%t\release\server
    set DEST_BINARY=%ASSETS_BIN_DIR%\!BINARY_NAME!
    
    if exist "!SOURCE_BINARY!" (
        copy "!SOURCE_BINARY!" "!DEST_BINARY!" >nul
        echo [OK] Binaire !BINARY_NAME! copié vers assets
    ) else (
        echo [ERREUR] Binaire non trouvé pour %%t : !SOURCE_BINARY!
    )
    
    cd ..\platforms\android
    
    :next_target
)

echo.
echo ========================================
echo Binaires compilés et copiés dans assets:
if exist "%ASSETS_BIN_DIR%" (
    dir /b "%ASSETS_BIN_DIR%"
) else (
    echo Aucun binaire trouvé
)
echo ========================================
echo.
echo COMPILATION TERMINÉE !
echo.
echo Vous pouvez maintenant exécuter:
echo   gradlew assembleDebug
echo.

pause
