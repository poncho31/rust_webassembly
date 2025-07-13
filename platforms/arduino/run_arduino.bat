@echo off
setlocal enabledelayedexpansion
echo Arduino Deployer - Simple Arduino Code Deployment Tool
echo =======================================================

:: Check if arduino-cli is installed
set "LOCAL_ARDUINO_CLI=%~dp0cli\arduino-cli.exe"
if exist "%LOCAL_ARDUINO_CLI%" (
    echo ✅ Using local Arduino CLI: %LOCAL_ARDUINO_CLI%
    set "ARDUINO_CLI=%LOCAL_ARDUINO_CLI%"
) else (
    where arduino-cli >nul 2>&1
    if %errorlevel% neq 0 (
        echo.
        echo ❌ Arduino CLI not found!
        echo Please place arduino-cli.exe in: %~dp0cli\arduino-cli.exe
        echo Or install it system-wide from: https://arduino.github.io/arduino-cli/
        echo.
        pause
        exit /b 1
    ) else (
        echo ✅ Using system Arduino CLI
        set "ARDUINO_CLI=arduino-cli"
    )
)

:: Initialize Arduino CLI if needed
echo Checking Arduino CLI configuration...
%ARDUINO_CLI% config init >nul 2>&1
%ARDUINO_CLI% core update-index >nul 2>&1
%ARDUINO_CLI% core install arduino:avr >nul 2>&1
echo ✅ Arduino CLI ready!

:: Build the project
echo Building Arduino deployer...
cargo build --release
if %errorlevel% neq 0 (
    echo ERROR: Failed to build project
    pause
    exit /b 1
)

:: Set executable path
set "ARDUINO_EXE=..\..\target\release\arduino-deploy.exe"

:: Run the application
echo.
echo Available commands:
echo   list                   - List available serial ports
echo   deploy                 - Deploy a sketch to Arduino
echo   monitor                - Monitor serial output
echo   example                - Create example sketch
echo   boards                 - Show supported boards
echo.

set /p choice=Enter command 

if /i "%choice%"=="list" (
    %ARDUINO_EXE% list
) else if /i "%choice%"=="deploy" (
    echo.
    echo Available ports:
    %ARDUINO_EXE% list
    echo.
    echo Available sketches:
    if exist "examples\*.ino" (
        echo   Examples directory:
        for %%f in (examples\*.ino) do echo     - %%f
    )
    if exist "*.ino" (
        echo   Current directory:
        for %%f in (*.ino) do echo     - %%f
    )
    echo.
    echo 💡 You can now organize your sketches however you want:
    echo   - Put them directly in examples\ folder
    echo   - Create subfolders in examples\ 
    echo   - Put them anywhere and specify the full path
    echo   - No more stupid Arduino CLI folder naming constraints!
    echo.
    echo Examples:
    echo   - examples\blink.ino (LED blink)
    echo   - examples\dht_sensor.ino (Temperature sensor)
    echo   - examples\ultrasonic.ino (Distance sensor)
    echo   - examples\esp8266_wifi_test.ino (ESP8266 WiFi test)
    echo   - examples\esp8266_webserver.ino (ESP8266 web server)
    echo   - examples\esp8266_api_server.ino (ESP8266 REST API)
    echo.
    set /p sketch=Enter sketch path (.ino file) or press Enter for examples\blink.ino 
    if "%sketch%"=="" set sketch=examples\blink.ino
    set /p port=Enter port (or press Enter to auto-detect) 
    set /p board=Enter board type (uno/nano/mega/leonardo) or press Enter for nano 
    if "%board%"=="" set board=nano 
    if "%port%"=="" (
        echo Auto-detecting port...
        for /f "tokens=2 delims= " %%a in ('%ARDUINO_EXE% list ^| findstr "COM"') do set "port=%%a"
        if defined port (
            echo Using auto-detected port: !port!
        ) else (
            echo No port auto-detected, using COM3 as default
            set "port=COM3"
        )
    )
    %ARDUINO_EXE% deploy --sketch "%sketch%" --port "%port%" --board "%board%" --arduino-cli "%ARDUINO_CLI%"
) else if /i "%choice%"=="monitor" (
    echo.
    echo Available ports:
    %ARDUINO_EXE% list
    echo.
    set /p port=Enter port to monitor (or press Enter to auto-detect) 
    if "%port%"=="" (
        echo Auto-detecting port...
        for /f "tokens=2 delims= " %%a in ('%ARDUINO_EXE% list ^| findstr "COM"') do set "port=%%a"
        if defined port (
            echo Using auto-detected port: !port!
        ) else (
            echo No port auto-detected, using COM3 as default
            set "port=COM3"
        )
    )
    set /p baud=Enter baud rate (default 9600) 
    if "%baud%"=="" set baud=9600
    %ARDUINO_EXE% monitor --port "%port%" --baud "%baud%"
) else if /i "%choice%"=="example" (
    %ARDUINO_EXE% example
    echo Example sketch created at: examples\blink.ino
) else if /i "%choice%"=="boards" (
    %ARDUINO_EXE% boards
) else (
    echo Invalid choice. Please try again.
)

pause
