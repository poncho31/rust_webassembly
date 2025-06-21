@echo off
setlocal enabledelayedexpansion

REM Vérifier si l'option clean est demandée
if "%1"=="--clean" goto :clean

echo [INFO] ========================================
echo [INFO] Construction APK WebAssembly Unified
echo [INFO] ========================================

REM Vérifier les prérequis
echo [INFO] Vérification des prérequis...

REM Vérifier Rust
where rustc >nul 2>nul
if errorlevel 1 (
    echo [ERROR] Rust n'est pas installé
    exit /b 1
)

REM Forcer l'utilisation de Java JDK 17 local (pour compatibilité Gradle)
set "LOCAL_JDK=%~dp0jdk17"
echo [INFO] Configuration de Java JDK 17 local...

if not exist "%LOCAL_JDK%\jdk-17.0.2" (
    echo [INFO] Installation de Java JDK 17...
    
    echo [INFO] Téléchargement de Java JDK 17...
    powershell -Command "try { Invoke-WebRequest -Uri 'https://download.java.net/java/GA/jdk17.0.2/dfd4a8d0985749f896bed50d7138ee7f/8/GPL/openjdk-17.0.2_windows-x64_bin.zip' -OutFile 'jdk17.zip' -UseBasicParsing } catch { Write-Host 'Erreur de téléchargement:' $_.Exception.Message; exit 1 }"
    
    if errorlevel 1 (
        echo [ERROR] Échec du téléchargement de Java JDK 17
        exit /b 1
    )
    
    REM Extraire JDK localement
    if not exist "%LOCAL_JDK%" mkdir "%LOCAL_JDK%"
    powershell -Command "try { Expand-Archive -Path 'jdk17.zip' -DestinationPath '%LOCAL_JDK%' -Force } catch { Write-Host 'Erreur extraction:' $_.Exception.Message; exit 1 }"
    
    if errorlevel 1 (
        echo [ERROR] Échec de l'extraction de Java JDK 17
        exit /b 1
    )
    
    del jdk17.zip
    echo [INFO] Java JDK 17 installé localement
) else (
    echo [INFO] Java JDK 17 local trouvé
)

REM Configurer JAVA_HOME pour utiliser la version locale
for /d %%i in ("%LOCAL_JDK%\jdk-*") do set "JAVA_HOME=%%i"
set "PATH=%JAVA_HOME%\bin;%PATH%"
echo [INFO] Utilisation de Java local: %JAVA_HOME%

REM Vérifier que Java 17 est bien utilisé
java -version 2>&1 | findstr "17\." >nul
if errorlevel 1 (
    echo [ERROR] Java 17 n'est pas correctement configuré
    java -version
    exit /b 1
)
echo [INFO] Java 17 correctement configuré

REM Installer cargo-ndk localement
set "LOCAL_CARGO_HOME=%~dp0local-cargo"
set "CARGO_HOME=%LOCAL_CARGO_HOME%"
set "PATH=%LOCAL_CARGO_HOME%\bin;%PATH%"

if not exist "%LOCAL_CARGO_HOME%\bin\cargo-ndk.exe" (
    echo [INFO] Installation locale de cargo-ndk...
    if not exist "%LOCAL_CARGO_HOME%" mkdir "%LOCAL_CARGO_HOME%"
    cargo install --root "%LOCAL_CARGO_HOME%" cargo-ndk
) else (
    echo [INFO] cargo-ndk trouvé localement
)

REM Configurer Android SDK local
set "LOCAL_ANDROID_SDK=%~dp0android-sdk"
set "ANDROID_HOME=%LOCAL_ANDROID_SDK%"
set "PATH=%ANDROID_HOME%\cmdline-tools\latest\bin;%ANDROID_HOME%\platform-tools;%PATH%"

echo [INFO] Configuration Android SDK local: %LOCAL_ANDROID_SDK%

REM Installer Android SDK si nécessaire
if not exist "%LOCAL_ANDROID_SDK%\cmdline-tools" (
    echo [INFO] Installation d'Android SDK...
    
    REM Créer le dossier SDK
    if not exist "%LOCAL_ANDROID_SDK%" mkdir "%LOCAL_ANDROID_SDK%"
    cd /d "%LOCAL_ANDROID_SDK%"
    
    REM Télécharger command line tools
    echo [INFO] Téléchargement des Android Command Line Tools...
    powershell -Command "Invoke-WebRequest -Uri 'https://dl.google.com/android/repository/commandlinetools-win-9477386_latest.zip' -OutFile 'cmdline-tools.zip'"
    
    REM Extraire les outils
    echo [INFO] Extraction des outils...
    powershell -Command "Expand-Archive -Path 'cmdline-tools.zip' -DestinationPath '.' -Force"
    
    REM Réorganiser la structure des dossiers
    if not exist "cmdline-tools\latest" mkdir "cmdline-tools\latest"
    move cmdline-tools\bin cmdline-tools\latest\
    move cmdline-tools\lib cmdline-tools\latest\
    move cmdline-tools\NOTICE.txt cmdline-tools\latest\
    move cmdline-tools\source.properties cmdline-tools\latest\
    
    REM Nettoyer
    del cmdline-tools.zip
    
    echo [INFO] Android Command Line Tools installés
)

REM Mettre à jour PATH pour cette session
set PATH=%ANDROID_HOME%\cmdline-tools\latest\bin;%ANDROID_HOME%\platform-tools;%PATH%

REM Installer les composants SDK nécessaires
echo [INFO] Vérification des composants Android SDK...
if not exist "%ANDROID_HOME%\platform-tools" (
    echo [INFO] Acceptance des licences et installation des composants SDK...
    call accept_all_licenses.bat
    
    echo [INFO] Installation des composants SDK nécessaires...
    "%ANDROID_HOME%\cmdline-tools\latest\bin\sdkmanager" "platform-tools" "platforms;android-33" "build-tools;33.0.2" "ndk;25.2.9519653"
)

REM Configurer NDK
set ANDROID_NDK_ROOT=%ANDROID_HOME%\ndk\25.2.9519653
set NDK_HOME=%ANDROID_NDK_ROOT%

REM Ajouter les targets Android à Rust
echo [INFO] Configuration des targets Android...

REM Vérifier et ajouter les targets uniquement s'ils ne sont pas présents
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android
rustup target add i686-linux-android

REM Compiler le code Rust pour Android
echo [INFO] Compilation du code Rust pour Android...
cd /d "%~dp0"

REM Construire pour toutes les architectures Android
set CARGO_NDK_PATH=%LOCAL_CARGO_HOME%\bin\cargo-ndk.exe
if not exist "%CARGO_NDK_PATH%" (
    echo [ERROR] cargo-ndk n'est pas trouvé à %CARGO_NDK_PATH%
    exit /b 1
)

echo [INFO] Compilation pour aarch64-linux-android...
cargo ndk -t aarch64-linux-android -p 21 -- build --release

echo [INFO] Compilation pour armv7-linux-androideabi...
cargo ndk -t armv7-linux-androideabi -p 21 -- build --release

echo [INFO] Compilation pour x86_64-linux-android...
cargo ndk -t x86_64-linux-android -p 21 -- build --release

echo [INFO] Compilation pour i686-linux-android...
cargo ndk -t i686-linux-android -p 21 -- build --release

if errorlevel 1 (
    echo [ERROR] Échec de la compilation Rust
    exit /b 1
)

REM Copier les bibliothèques compilées
echo [INFO] Copie des bibliothèques natives...
set JNI_LIBS=app\src\main\jniLibs

if not exist "%JNI_LIBS%" mkdir "%JNI_LIBS%"
if not exist "%JNI_LIBS%\arm64-v8a" mkdir "%JNI_LIBS%\arm64-v8a"
if not exist "%JNI_LIBS%\armeabi-v7a" mkdir "%JNI_LIBS%\armeabi-v7a"
if not exist "%JNI_LIBS%\x86_64" mkdir "%JNI_LIBS%\x86_64"
if not exist "%JNI_LIBS%\x86" mkdir "%JNI_LIBS%\x86"

copy /Y "..\..\target\aarch64-linux-android\release\libwebassembly_android.so" "%JNI_LIBS%\arm64-v8a\"
copy /Y "..\..\target\armv7-linux-androideabi\release\libwebassembly_android.so" "%JNI_LIBS%\armeabi-v7a\"
copy /Y "..\..\target\x86_64-linux-android\release\libwebassembly_android.so" "%JNI_LIBS%\x86_64\"
copy /Y "..\..\target\i686-linux-android\release\libwebassembly_android.so" "%JNI_LIBS%\x86\"

REM Installer Gradle Wrapper JAR si nécessaire
echo [INFO] Vérification du Gradle Wrapper...
if not exist "gradle\wrapper\gradle-wrapper.jar" (
    echo [INFO] Téléchargement du Gradle Wrapper...
    if not exist "gradle\wrapper" mkdir "gradle\wrapper"
    powershell -Command "Invoke-WebRequest -Uri 'https://services.gradle.org/distributions/gradle-8.0-bin.zip' -OutFile 'gradle-8.0-bin.zip'"
    powershell -Command "Expand-Archive -Path 'gradle-8.0-bin.zip' -DestinationPath 'gradle-temp' -Force"
    copy /Y "gradle-temp\gradle-8.0\lib\gradle-wrapper.jar" "gradle\wrapper\"
    rmdir /s /q gradle-temp
    del gradle-8.0-bin.zip
    echo [INFO] Gradle Wrapper installé localement
) else (
    echo [INFO] Gradle Wrapper trouvé
)

REM Construire l'APK
echo [INFO] Construction de l'APK Android...
call gradlew assembleDebug

if errorlevel 1 (
    echo [ERROR] Échec de la construction de l'APK
    exit /b 1
)

echo [INFO] ========================================
echo [INFO] APK construit avec succès !
echo [INFO] Fichier: app\build\outputs\apk\debug\app-debug.apk
echo [INFO] ========================================

REM Optionnel : installer sur un appareil connecté
set /p INSTALL="Installer sur l'appareil connecté ? (y/n): "
if /i "%INSTALL%"=="y" (
    echo [INFO] Installation sur l'appareil...
    "%ANDROID_HOME%\platform-tools\adb" install -r "app\build\outputs\apk\debug\app-debug.apk"
)

pause
goto :eof

:clean
echo [INFO] ========================================
echo [INFO] Nettoyage complet de l'environnement Android
echo [INFO] ========================================

REM Supprimer tous les outils installés localement
if exist "local-cargo" (
    echo [INFO] Suppression de cargo local...
    rmdir /s /q local-cargo
)

if exist "local-rustup" (
    echo [INFO] Suppression de rustup local...
    rmdir /s /q local-rustup
)

if exist "android-sdk" (
    echo [INFO] Suppression d'Android SDK local...
    rmdir /s /q android-sdk
)

if exist "jdk17" (
    echo [INFO] Suppression de JDK local...
    rmdir /s /q jdk17
)

if exist "app\build" (
    echo [INFO] Suppression du build Android...
    rmdir /s /q app\build
)

if exist "app\src\main\jniLibs" (
    echo [INFO] Suppression des bibliothèques JNI...
    rmdir /s /q app\src\main\jniLibs
)

if exist ".gradle" (
    echo [INFO] Suppression du cache Gradle...
    rmdir /s /q .gradle
)

if exist "build" (
    echo [INFO] Suppression du dossier build...
    rmdir /s /q build
)

REM Supprimer les fichiers temporaires
del /q *.zip 2>nul
del /q *.tmp 2>nul
del /q *.log 2>nul

echo [INFO] ========================================
echo [INFO] Nettoyage terminé ! Vous pouvez relancer build_android.bat
echo [INFO] ========================================
pause
