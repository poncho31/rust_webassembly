@echo off
echo === BUILD ANDROID APK avec DOCKER (Windows) ===
echo.

cd /d "%~dp0"

REM Vérification d'ADB
echo 1. Verification d'ADB...
where adb >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [INFO] ADB non trouve. Installation automatique...
    
    REM Créer le dossier platform-tools si nécessaire
    if not exist "platform-tools" (
        echo [INFO] Telechargement des Android Platform Tools...
        powershell -Command "Invoke-WebRequest -Uri 'https://dl.google.com/android/repository/platform-tools-latest-windows.zip' -OutFile 'platform-tools.zip'"
        powershell -Command "Expand-Archive -Path 'platform-tools.zip' -DestinationPath '.'"
        del platform-tools.zip
    )
    
    REM Ajouter platform-tools au PATH temporairement
    set "PATH=%CD%\platform-tools;%PATH%"
    echo    ✓ ADB installe automatiquement
) else (
    echo    ✓ ADB trouve
)

adb version | findstr "Android Debug Bridge"
echo.

REM Configuration du serveur ADB
echo 2. Configuration du serveur ADB...
echo    Arret du serveur ADB...
adb kill-server 2>nul

echo    Demarrage du serveur ADB...
adb start-server

REM Attendre un peu pour que le serveur démarre
timeout 2 >nul

echo    ✓ Serveur ADB configure
echo.

REM Vérification du téléphone
echo 3. Verification du telephone...
adb devices | findstr /C:"device" | findstr /V /C:"List of devices" >nul
if %ERRORLEVEL% neq 0 (
    echo [ATTENTION] Aucun telephone detecte !
    echo.
    echo Verifiez :
    echo 1. Telephone branche en USB
    echo 2. Debogage USB active
    echo 3. Autorisation du debogage accordee
    echo.
    echo Appareils actuels :
    adb devices
    echo.
    
    :CHECK_DEVICE_LOOP
    echo Attendre la connexion du telephone... (Ctrl+C pour annuler)
    timeout 3 >nul
    adb devices | findstr /C:"device" | findstr /V /C:"List of devices" >nul
    if %ERRORLEVEL% neq 0 goto CHECK_DEVICE_LOOP
    
    echo    ✓ Telephone detecte !
) else (
    echo    ✓ Telephone detecte
)

adb devices | findstr /V /C:"List of devices"
echo.

REM Construction de l'image Docker
echo 4. Construction de l'image Docker...
echo    Reconstruction avec cache nettoye pour eviter les problemes de signature...
echo    Utilisation du target_android pour la compilation Android...
docker build --no-cache -f Dockerfile.android -t webassembly-android ../..
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Echec de la construction de l'image Docker !
    pause
    exit /b 1
)
echo    ✓ Image Docker construite
echo.

REM Lancement du conteneur en mode TCP (Windows)
echo 5. Lancement du build Android via docker-compose...
echo    Mode : Détection automatique (TCP sur Windows)
echo    Le conteneur va se connecter au serveur ADB de votre PC
echo.

set ADB_MODE=tcp
set ADB_TCP_HOST=host.docker.internal

docker-compose up --build

if %ERRORLEVEL% neq 0 (
    echo.
    echo [ERROR] Le build Docker a echoue !
    echo.
    echo Diagnostic :
    echo 1. Verifiez que votre telephone est toujours connecte : adb devices
    echo 2. Redemarrez le serveur ADB : adb kill-server ^&^& adb start-server
    echo 3. Relancez ce script
    echo.
    pause
    exit /b 1
)

echo.
echo === BUILD TERMINE AVEC SUCCES ===
echo L'APK a ete genere, installe et l'application est lancee !
echo Les logs s'affichent en continu. Appuyez sur Ctrl+C pour arreter.
pause
