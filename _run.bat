@echo off
setlocal EnableDelayedExpansion

REM Charger les variables du .env dans l'environnement, en ignorant les lignes vides et les commentaires
for /f "usebackq tokens=1,* delims==" %%A in (".env") do (
    set "line=%%A"
    echo !line! | findstr /b /r /c:"[a-zA-Z_]" >nul
    if not errorlevel 1 (
        set "%%A=%%B"
    )
)

REM Vérifie si le premier argument est 'docker'
if /i "%1"=="docker" (
    docker-compose down
    docker-compose up -d

    start "" %ALLOWED_ORIGIN_DOCKER%
    docker logs -f %APP_NAME_DOCKER%

) else if /i "%1"=="android" (
    if /i "%2"=="docker" (
        echo [INFO] Lancement de la construction Android avec Docker...
        cd platforms\android
        call build_android_docker.bat
        cd ..\..
    ) else (
        echo [INFO] Construction de l'APK Android...
        cd platforms\android
        call build_android.bat
        cd ..\..
    )

) else if /i "%1"=="arduino" (
    if /i "%2"=="esp32" (
        echo [INFO] Déploiement Arduino ESP32...
        echo [WARN] Support ESP32 en cours de développement
        cd platforms\arduino
        REM TODO: Ajouter support ESP32
        cargo run --release -- complete
        cd ..\..
    ) else (
        echo [INFO] Déploiement Arduino ESP8266...
        cd platforms\arduino
        cargo run --release -- complete
        cd ..\..
    )

) else (
    REM Vérifie la présence de cargo et installe Rust si manquant
    where cargo >nul 2>nul
    if errorlevel 1 (
        echo [INFO] cargo non trouvé, installation de Rust...
        powershell -Command "Invoke-WebRequest -Uri https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe -OutFile rustup-init.exe; Start-Process -Wait .\rustup-init.exe -ArgumentList '-y'; Remove-Item .\rustup-init.exe"
        refreshenv
    )

    REM Vérifie la présence de wasm-pack et installe si manquant
    where wasm-pack >nul 2>nul
    if errorlevel 1 (
        echo [INFO] wasm-pack non trouvé, installation...
        cargo install wasm-pack
    )

    REM Vérifie la présence de cargo-ndk et installe si manquant
    where cargo-ndk >nul 2>nul
    if errorlevel 1 (
        echo [INFO] cargo-ndk non trouvé, installation...
        cargo install cargo-ndk
    )

    if /i "%1"=="force" (
        taskkill /f /im rust-analyzer.exe
        taskkill /f /im server.exe

        cargo clean
        del /q /s client\static\pkg\*

        echo [INFO] Projet nettoyé.
    )

    REM Compilation WebAssembly
    cd client
    wasm-pack build --target web --out-dir static/pkg
    cd ..

    REM Compilation Rust
    cargo build --release

    REM Lancement du navigateur
    start "" %ALLOWED_ORIGIN%

    REM Lancement du serveur
    target\release\server.exe
)

pause
