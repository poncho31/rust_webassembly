@echo off
echo ===============================================================================
echo                      WEBASSEMBLY UNIFIED - ANDROID TEST
echo ===============================================================================

echo [1/5] Checking ADB connection...
adb devices
if errorlevel 1 (
    echo [ERROR] ADB not found or no devices connected!
    echo [INFO] Make sure Android SDK is installed and device is connected with USB debugging enabled.
    pause
    exit /b 1
)

echo.
echo [2/5] Installing APK...
adb install -r app\build\outputs\apk\debug\app-debug.apk
if errorlevel 1 (
    echo [ERROR] Failed to install APK!
    pause
    exit /b 1
)

echo.
echo [3/5] Clearing logs...
adb logcat -c

echo.
echo [4/5] Starting app...
adb shell am start -n com.main/.MainActivity

echo.
echo [5/5] Monitoring logs for CORS and server issues...
echo [INFO] Look for the following key indicators:
echo   - "Rust initialization result: true" = Server started successfully
echo   - "Server URL received: http://127.0.0.1:8088" = Correct URL
echo   - "Loading WebView with server URL" = No CORS fallback
echo   - NO "file:///android_asset" messages = CORS issue avoided
echo   - "Console: ✅ Android détecté" = JavaScript running properly
echo.
echo [INFO] Press Ctrl+C to stop monitoring...
echo.
adb logcat -s rust_webassembly_android