@echo off
setlocal enabledelayedexpansion

REM =============================================================================
REM                     WEBASSEMBLY UNIFIED ANDROID BUILD SCRIPT
REM =============================================================================
REM This script builds an Android APK for the WebAssembly Unified project.
REM Usage: build_android.bat [--clean]
REM =============================================================================

echo.
echo ===============================================================================
echo                     WEBASSEMBLY UNIFIED - ANDROID BUILD
echo ===============================================================================
echo.

REM Check if clean option is requested
if "%1"=="--clean" goto :clean

echo [STEP 1/8] Checking prerequisites...
echo ---------------------------------------------------------------------------

REM Check Rust installation
where rustc >nul 2>nul
if errorlevel 1 (
    echo [ERROR] Rust is not installed. Please install Rust first.
    echo         Visit: https://rustup.rs/
    pause
    exit /b 1
)
echo [OK] Rust found: 
rustc --version

echo.
echo [STEP 2/8] Setting up Java JDK 17...
echo ---------------------------------------------------------------------------

REM Force the use of local Java JDK 17 (for Gradle compatibility)
set "LOCAL_JDK=%~dp0jdk17"

if not exist "%LOCAL_JDK%\jdk-17.0.2" (
    echo [INFO] Installing Java JDK 17 locally...
    
    echo     * Downloading Java JDK 17...
    powershell -Command "try { Invoke-WebRequest -Uri 'https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_windows-x64_bin.zip' -OutFile 'jdk17.zip' -UseBasicParsing } catch { Write-Host 'Download error:' $_.Exception.Message; exit 1 }"
    
    if errorlevel 1 (
        echo [ERROR] Failed to download Java JDK 17
        pause
        exit /b 1
    )
    
    REM Extract JDK locally
    if not exist "%LOCAL_JDK%" mkdir "%LOCAL_JDK%"
    echo     * Extracting JDK...
    powershell -Command "try { Expand-Archive -Path 'jdk17.zip' -DestinationPath '%LOCAL_JDK%' -Force } catch { Write-Host 'Extraction error:' $_.Exception.Message; exit 1 }"
    
    if errorlevel 1 (
        echo [ERROR] Failed to extract Java JDK 17
        pause
        exit /b 1
    )
    
    del jdk17.zip
    echo [OK] Java JDK 17 installed locally
) else (
    echo [OK] Local Java JDK 17 found
)

REM Configure JAVA_HOME to use local version
for /d %%i in ("%LOCAL_JDK%\jdk-*") do set "JAVA_HOME=%%i"
set "PATH=%JAVA_HOME%\bin;%PATH%"
echo [INFO] Using local Java: %JAVA_HOME%

REM Verify that Java 17 is being used
java -version 2>&1 | findstr "17\." >nul
if errorlevel 1 (
    echo [ERROR] Java 17 is not properly configured
    java -version
    pause
    exit /b 1
)
echo [OK] Java 17 properly configured

echo.
echo [STEP 3/8] Setting up cargo-ndk...
echo ---------------------------------------------------------------------------
REM Install cargo-ndk locally
set "LOCAL_CARGO_HOME=%~dp0local-cargo"
set "CARGO_HOME=%LOCAL_CARGO_HOME%"
set "PATH=%LOCAL_CARGO_HOME%\bin;%PATH%"

if not exist "%LOCAL_CARGO_HOME%\bin\cargo-ndk.exe" (
    echo [INFO] Installing cargo-ndk locally...
    if not exist "%LOCAL_CARGO_HOME%" mkdir "%LOCAL_CARGO_HOME%"
    cargo install --root "%LOCAL_CARGO_HOME%" cargo-ndk
    if errorlevel 1 (
        echo [ERROR] Failed to install cargo-ndk
        pause
        exit /b 1
    )
    echo [OK] cargo-ndk installed locally
) else (
    echo [OK] cargo-ndk found locally
)

echo.
echo [STEP 4/8] Setting up Android SDK...
echo ---------------------------------------------------------------------------
REM Configure local Android SDK
set "LOCAL_ANDROID_SDK=%~dp0android-sdk"
set "ANDROID_HOME=%LOCAL_ANDROID_SDK%"
set "PATH=%ANDROID_HOME%\cmdline-tools\latest\bin;%ANDROID_HOME%\platform-tools;%PATH%"

echo [INFO] Android SDK location: %LOCAL_ANDROID_SDK%

REM Install Android SDK if necessary
if not exist "%LOCAL_ANDROID_SDK%\cmdline-tools" (
    echo [INFO] Installing Android SDK...
    
    REM Create SDK folder
    if not exist "%LOCAL_ANDROID_SDK%" mkdir "%LOCAL_ANDROID_SDK%"
    cd /d "%LOCAL_ANDROID_SDK%"
    
    REM Download command line tools
    echo     * Downloading Android Command Line Tools...
    powershell -Command "Invoke-WebRequest -Uri 'https://dl.google.com/android/repository/commandlinetools-win-9477386_latest.zip' -OutFile 'cmdline-tools.zip'"
    
    if errorlevel 1 (
        echo [ERROR] Failed to download Android Command Line Tools
        pause
        exit /b 1
    )
    
    REM Extract tools
    echo     * Extracting tools...
    powershell -Command "Expand-Archive -Path 'cmdline-tools.zip' -DestinationPath '.' -Force"
    
    REM Reorganize folder structure
    if not exist "cmdline-tools\latest" mkdir "cmdline-tools\latest"
    move cmdline-tools\bin cmdline-tools\latest\
    move cmdline-tools\lib cmdline-tools\latest\
    move cmdline-tools\NOTICE.txt cmdline-tools\latest\
    move cmdline-tools\source.properties cmdline-tools\latest\
    
    REM Clean up
    del cmdline-tools.zip
    
    echo [OK] Android Command Line Tools installed
) else (
    echo [OK] Android SDK found
)

REM Update PATH for this session
set PATH=%ANDROID_HOME%\cmdline-tools\latest\bin;%ANDROID_HOME%\platform-tools;%PATH%

REM Install necessary SDK components
echo [INFO] Checking Android SDK components...
if not exist "%ANDROID_HOME%\platform-tools" (
    echo [INFO] Accepting licenses and installing SDK components...
    call accept_all_licenses.bat
      echo [INFO] Installing necessary SDK components...
    "%ANDROID_HOME%\cmdline-tools\latest\bin\sdkmanager" "platform-tools" "platforms;android-34" "build-tools;34.0.0" "ndk;25.2.9519653"
    
    if errorlevel 1 (
        echo [ERROR] Failed to install SDK components
        pause
        exit /b 1
    )
    echo [OK] SDK components installed
) else (
    echo [OK] SDK components found
)

REM Configure NDK
set ANDROID_NDK_ROOT=%ANDROID_HOME%\ndk\25.2.9519653
set NDK_HOME=%ANDROID_NDK_ROOT%
echo [INFO] NDK location: %ANDROID_NDK_ROOT%

echo.
echo [STEP 5/8] Adding Android targets to Rust...
echo ---------------------------------------------------------------------------

REM Check and add targets only if they are not present
echo [INFO] Adding Android targets:
echo     * aarch64-linux-android (ARM64)
rustup target add aarch64-linux-android
echo     * armv7-linux-androideabi (ARM32)
rustup target add armv7-linux-androideabi
echo     * x86_64-linux-android (x64)
rustup target add x86_64-linux-android
echo     * i686-linux-android (x86)
rustup target add i686-linux-android

echo.
echo [STEP 6/8] Compiling Rust code for Android...
echo ---------------------------------------------------------------------------
cd /d "%~dp0"

REM Configure Android-specific target directory
set ANDROID_TARGET_DIR=%~dp0..\..\target_android
echo [INFO] Using Android-specific target directory: %ANDROID_TARGET_DIR%

REM Build for all Android architectures
set CARGO_NDK_PATH=%LOCAL_CARGO_HOME%\bin\cargo-ndk.exe
if not exist "%CARGO_NDK_PATH%" (
    echo [ERROR] cargo-ndk not found at %CARGO_NDK_PATH%
    pause
    exit /b 1
)

echo [INFO] Building for multiple architectures...

echo     * Building for aarch64-linux-android (ARM64)...
cargo ndk -t aarch64-linux-android -p 21 -- build --release --target-dir "%ANDROID_TARGET_DIR%"
if errorlevel 1 (
    echo [ERROR] Failed to compile for aarch64-linux-android
    pause
    exit /b 1
)

echo     * Building for armv7-linux-androideabi (ARM32)...
cargo ndk -t armv7-linux-androideabi -p 21 -- build --release --target-dir "%ANDROID_TARGET_DIR%"
if errorlevel 1 (
    echo [ERROR] Failed to compile for armv7-linux-androideabi
    pause
    exit /b 1
)

echo     * Building for x86_64-linux-android (x64)...
cargo ndk -t x86_64-linux-android -p 21 -- build --release --target-dir "%ANDROID_TARGET_DIR%"
if errorlevel 1 (
    echo [ERROR] Failed to compile for x86_64-linux-android
    pause
    exit /b 1
)

echo     * Building for i686-linux-android (x86)...
cargo ndk -t i686-linux-android -p 21 -- build --release --target-dir "%ANDROID_TARGET_DIR%"
if errorlevel 1 (
    echo [ERROR] Failed to compile for i686-linux-android
    pause
    exit /b 1
)

echo [OK] All architectures compiled successfully

echo.
echo [STEP 7/8] Copying native libraries and static files...
echo ---------------------------------------------------------------------------
REM Copy compiled libraries
echo [INFO] Copying native libraries to JNI folders...
set JNI_LIBS=app\src\main\jniLibs

if not exist "%JNI_LIBS%" mkdir "%JNI_LIBS%"
if not exist "%JNI_LIBS%\arm64-v8a" mkdir "%JNI_LIBS%\arm64-v8a"
if not exist "%JNI_LIBS%\armeabi-v7a" mkdir "%JNI_LIBS%\armeabi-v7a"
if not exist "%JNI_LIBS%\x86_64" mkdir "%JNI_LIBS%\x86_64"
if not exist "%JNI_LIBS%\x86" mkdir "%JNI_LIBS%\x86"

echo     * Copying ARM64 library...
copy /Y "%ANDROID_TARGET_DIR%\aarch64-linux-android\release\libwebassembly_android.so" "%JNI_LIBS%\arm64-v8a\"
echo     * Copying ARM32 library...
copy /Y "%ANDROID_TARGET_DIR%\armv7-linux-androideabi\release\libwebassembly_android.so" "%JNI_LIBS%\armeabi-v7a\"
echo     * Copying x64 library...
copy /Y "%ANDROID_TARGET_DIR%\x86_64-linux-android\release\libwebassembly_android.so" "%JNI_LIBS%\x86_64\"
echo     * Copying x86 library...
copy /Y "%ANDROID_TARGET_DIR%\i686-linux-android\release\libwebassembly_android.so" "%JNI_LIBS%\x86\"

if errorlevel 1 (
    echo [ERROR] Failed to copy native libraries
    pause
    exit /b 1
)
echo [OK] Native libraries copied successfully

REM Install Gradle Wrapper JAR if necessary
echo [INFO] Checking Gradle Wrapper...
if not exist "gradle\wrapper\gradle-wrapper.jar" (
    echo [INFO] Downloading Gradle Wrapper...
    if not exist "gradle\wrapper" mkdir "gradle\wrapper"
    powershell -Command "Invoke-WebRequest -Uri 'https://services.gradle.org/distributions/gradle-8.5-bin.zip' -OutFile 'gradle-8.5-bin.zip'"
    powershell -Command "Expand-Archive -Path 'gradle-8.5-bin.zip' -DestinationPath 'gradle-temp' -Force"
    copy /Y "gradle-temp\gradle-8.5\lib\gradle-wrapper.jar" "gradle\wrapper\"
    rmdir /s /q gradle-temp
    del gradle-8.5-bin.zip
    echo [OK] Gradle Wrapper installed locally
) else (
    echo [OK] Gradle Wrapper found
)

echo.
echo [INFO] Copying client static files...
echo     ---------------------------------------------------------------

REM Variables for static files
set SOURCE_STATIC_DIR=%~dp0..\..\client\static
set ASSETS_STATIC_DIR=%~dp0app\src\main\assets\static
set ASSETS_PKG_DIR=%~dp0app\src\main\assets\static\pkg

echo [INFO] Source: %SOURCE_STATIC_DIR%
echo [INFO] Destination: %ASSETS_STATIC_DIR%

REM Create assets directories if they don't exist
if not exist "%ASSETS_STATIC_DIR%" (
    mkdir "%ASSETS_STATIC_DIR%"
    echo [INFO] Created assets/static directory
)

if not exist "%ASSETS_PKG_DIR%" (
    mkdir "%ASSETS_PKG_DIR%"
    echo [INFO] Created assets/static/pkg directory
)

REM Remove old files
echo [INFO] Cleaning old files...
del /q /s "%ASSETS_STATIC_DIR%\*" 2>nul
rmdir /s /q "%ASSETS_STATIC_DIR%" 2>nul
mkdir "%ASSETS_STATIC_DIR%"
mkdir "%ASSETS_PKG_DIR%"

REM Copy all static files
echo [INFO] Copying static files...
xcopy /E /I /Y "%SOURCE_STATIC_DIR%\*" "%ASSETS_STATIC_DIR%\"

if errorlevel 1 (
    echo [ERROR] Failed to copy static files
    pause
    exit /b 1
)

REM Check if WebAssembly pkg directory exists and copy it
set WASM_PKG_SOURCE=%~dp0..\..\client\static\pkg
if exist "%WASM_PKG_SOURCE%" (
    echo [INFO] Copying WebAssembly pkg files...
    xcopy /E /I /Y "%WASM_PKG_SOURCE%\*" "%ASSETS_PKG_DIR%\"
    if errorlevel 1 (
        echo [WARNING] Failed to copy WebAssembly pkg files
    ) else (
        echo [OK] WebAssembly pkg files copied
        echo [INFO] WebAssembly files in pkg:
        dir /b "%ASSETS_PKG_DIR%"
    )
) else (
    echo [INFO] No WebAssembly pkg directory found - may need to build WebAssembly first
)

echo [OK] Static files copied successfully
echo [INFO] Files copied to assets/static:
dir /b "%ASSETS_STATIC_DIR%"

echo.
echo [STEP 8/8] Building Android APK...
echo ---------------------------------------------------------------------------
echo [INFO] Building Android APK...
call gradlew assembleDebug

if errorlevel 1 (
    echo [ERROR] Failed to build APK
    pause
    exit /b 1
)

echo.
echo [OK] APK built successfully!
echo [INFO] File location: app\build\outputs\apk\debug\app-debug.apk
echo ===============================================================================

echo.
echo [INFO] Installing on connected device...
echo ---------------------------------------------------------------------------
"%ANDROID_HOME%\platform-tools\adb" install -r "app\build\outputs\apk\debug\app-debug.apk"

if errorlevel 1 (
    echo [WARNING] Installation failed or no device connected
) else (
    echo [OK] APK installed successfully
)


echo [INFO] Starting application...
adb shell am start -n com.main/.MainActivity

echo.
echo [INFO] Connected device(s):
adb devices

echo.
echo [INFO] Monitoring logs (Ctrl+C to stop)...
echo ---------------------------------------------------------------------------
adb logcat -c
adb logcat -s rust_webassembly_android

goto :eof


:clean
echo.
echo ===============================================================================
echo                        CLEAN ANDROID ENVIRONMENT
echo ===============================================================================
echo [INFO] Performing complete cleanup of Android environment...
echo.

REM Remove all locally installed tools
if exist "local-cargo" (
    echo [INFO] Removing local cargo...
    rmdir /s /q local-cargo
    echo [OK] Local cargo removed
)

if exist "local-rustup" (
    echo [INFO] Removing local rustup...
    rmdir /s /q local-rustup
    echo [OK] Local rustup removed
)

if exist "android-sdk" (
    echo [INFO] Removing local Android SDK...
    rmdir /s /q android-sdk
    echo [OK] Local Android SDK removed
)

if exist "jdk17" (
    echo [INFO] Removing local JDK...
    rmdir /s /q jdk17
    echo [OK] Local JDK removed
)

if exist "app\build" (
    echo [INFO] Removing Android build artifacts...
    rmdir /s /q app\build
    echo [OK] Android build artifacts removed
)

if exist "app\src\main\jniLibs" (
    echo [INFO] Removing JNI libraries...
    rmdir /s /q app\src\main\jniLibs
    echo [OK] JNI libraries removed
)

if exist "app\src\main\assets" (
    echo [INFO] Removing assets...
    rmdir /s /q app\src\main\assets
    echo [OK] Assets removed
)

if exist "..\..\target_android" (
    echo [INFO] Removing Android target directory...
    rmdir /s /q "..\..\target_android"
    echo [OK] Android target directory removed
)

if exist ".gradle" (
    echo [INFO] Removing Gradle cache...
    rmdir /s /q .gradle
    echo [OK] Gradle cache removed
)

if exist "build" (
    echo [INFO] Removing build folder...
    rmdir /s /q build
    echo [OK] Build folder removed
)

REM Remove temporary files
echo [INFO] Removing temporary files...
del /q *.zip 2>nul
del /q *.tmp 2>nul
del /q *.log 2>nul
echo [OK] Temporary files removed

echo.
echo ===============================================================================
echo                           CLEANUP COMPLETE!
echo ===============================================================================
echo [INFO] Environment cleaned successfully!
echo [INFO] You can now run build_android.bat again for a fresh build
echo ===============================================================================
pause
