@echo off
REM Charger les variables du .env dans l'environnement
for /f "usebackq tokens=1,2 delims==" %%A in (".env") do (
    if not "%%A"=="" set %%A=%%B
)


REM Vérifie si le premier argument est 'docker'
if /i "%1"=="docker" (

    docker-compose down
    docker-compose up -d

    start "" %ALLOWED_ORIGIN_DOCKER%
    docker logs -f %APP_NAME_DOCKER%

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
    
    REM compile webassembly
    cd client
    wasm-pack build --target web --out-dir static/pkg

    REM compile le projet Rust
    cd ..
    cargo build --release

    REM lancer la fenetre du navigateur
    start "" %ALLOWED_ORIGIN%

    REM Démarrer le serveur
    target\release\server.exe
)

pause

