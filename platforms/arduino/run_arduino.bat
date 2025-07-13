@echo off
title Arduino ESP8266 Auto Deployer
color 0A
echo.
echo ===============================================
echo     Arduino ESP8266 Auto Deployer
echo ===============================================
echo.
    echo Available sketches:

REM Default values
set SKETCH=examples\esp8266_simple_complete.ino
set PORT=COM5
set BOARD=esp8266:esp8266:nodemcuv2

REM Check command line arguments
if "%~1"=="--help" (
    echo Usage: run_arduino.bat [sketch] [port] [board]
    echo.
    echo Default values:
    echo   Sketch: %SKETCH%
    echo   Port: %PORT%  
    echo   Board: %BOARD%
    echo.
    echo Examples:
    echo   run_arduino.bat
    echo   run_arduino.bat examples\blink.ino COM3 uno
    echo   run_arduino.bat examples\esp8266_simple_complete.ino COM5
    echo   run_arduino.bat examples\esp8266_complete.ino COM7
    echo.
    pause
    exit /b 0
)

if not "%~1"=="" set SKETCH=%~1
if not "%~2"=="" set PORT=%~2
if not "%~3"=="" set BOARD=%~3

echo Configuration:
echo   Sketch: %SKETCH%
echo   Port: %PORT%
echo   Board: %BOARD%
echo.

REM Check if sketch file exists
if not exist "%SKETCH%" (
    echo ❌ Sketch file not found: %SKETCH%
    echo.
    echo Available sketches:
    dir /b examples\*.ino 2>nul
    echo.
    pause
    exit /b 1
)

REM Auto-detect ESP8266 deployment based on sketch filename
echo %SKETCH% | findstr /i "esp8266" >nul
if %errorlevel%==0 (
    echo 🔄 ESP8266 sketch detected - Using auto-deployment with web interface
    echo    Setting board to esp8266:esp8266:nodemcuv2
    set BOARD=esp8266:esp8266:nodemcuv2
    echo.
    cargo run --bin arduino-deploy -- deploy-web --sketch %SKETCH% --port %PORT% --board %BOARD% --arduino-cli ./cli/arduino-cli.exe
) else (
    echo 🔄 Standard Arduino deployment
    echo    Using board: %BOARD%
    echo.
    cargo run --bin arduino-deploy -- deploy --sketch %SKETCH% --port %PORT% --board %BOARD% --arduino-cli ./cli/arduino-cli.exe
)

if %errorlevel%==0 (
    echo.
    echo ✅ Deployment completed successfully!
    echo 🌐 If ESP8266, web interface should be open in your browser
    echo.
    echo 📡 Starting serial monitor to see Arduino output...
    echo Press Ctrl+C to stop monitoring
    echo.
    timeout /t 2 /nobreak >nul
    
    REM Set baud rate based on board type
    echo Debug: Board type is '%BOARD%'
    if "%BOARD%"=="esp8266:esp8266:nodemcuv2" (
        echo Using ESP8266 baud rate: 115200
        cargo run --bin arduino-deploy -- monitor --port %PORT% --baud 115200
    ) else (
        echo Using Arduino baud rate: 9600
        cargo run --bin arduino-deploy -- monitor --port %PORT% --baud 9600
    )
) else (
    echo.
    echo ❌ Deployment failed!
    echo.
)

echo Press any key to exit...
pause >nul
