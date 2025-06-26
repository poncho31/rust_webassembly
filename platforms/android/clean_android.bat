@echo off
echo === NETTOYAGE ANDROID - DESINSTALLATION DE L'APK ===
echo.

cd /d "%~dp0"

REM Vérification d'ADB
where adb >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [ERROR] ADB non trouve !
    echo Lancez d'abord build_android_docker.bat pour installer ADB
    pause
    exit /b 1
)

echo Verification des appareils connectes...
adb devices

REM Vérification du téléphone
adb devices | findstr /C:"device" | findstr /V /C:"List of devices" >nul
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Aucun telephone detecte !
    echo Branchez votre telephone et activez le debogage USB
    pause
    exit /b 1
)

echo.
echo Appareil detecte. Desinstallation de com.webassembly.unified...
adb uninstall com.webassembly.unified

if %ERRORLEVEL% equ 0 (
    echo ✓ Application desinstallee avec succes !
) else (
    echo ⚠ Application non trouvee ou deja desinstallee
)

echo.
echo === NETTOYAGE TERMINE ===
echo Vous pouvez maintenant relancer build_android_docker.bat
pause
