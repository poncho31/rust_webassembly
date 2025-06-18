@echo off
REM Charger les variables du .env dans l'environnement
for /f "usebackq tokens=1,2 delims==" %%A in (".env") do (
    if not "%%A"=="" set %%A=%%B
)

start "" https://%SERVER_HOST%:%SERVER_PORT%
target\release\server.exe
