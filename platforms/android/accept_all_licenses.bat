@echo off
setlocal

REM Variables d'environnement
set ANDROID_SDK_ROOT=%~dp0android-sdk
set ANDROID_HOME=%ANDROID_SDK_ROOT%

echo Accepting all Android SDK licenses...

REM Créer le répertoire de licences s'il n'existe pas
if not exist "%ANDROID_SDK_ROOT%\licenses" mkdir "%ANDROID_SDK_ROOT%\licenses"

REM Accepter toutes les licences connues en créant les fichiers appropriés
echo 8933bad161af4178b1185d1a37fbf41ea5269c55 > "%ANDROID_SDK_ROOT%\licenses\android-sdk-license"
echo d56f5187479451eabf01fb78af6dfcb131a6481e >> "%ANDROID_SDK_ROOT%\licenses\android-sdk-license"
echo 24333f8a63b6825ea9c5514f83c2829b004d1fee >> "%ANDROID_SDK_ROOT%\licenses\android-sdk-license"

echo 84831b9409646a918e30573bab4c9c91346d8abd > "%ANDROID_SDK_ROOT%\licenses\android-sdk-preview-license"

echo e9acab5b5fbb560a72cfaecce8946896ff6aab9d > "%ANDROID_SDK_ROOT%\licenses\android-googletv-license"

echo 601085b94cd77f0b54ff86406957099ebe79c4d6 > "%ANDROID_SDK_ROOT%\licenses\android-sdk-arm-dbt-license"

echo 859f317696f67ef3d7f30a50a5560e7834b43903 > "%ANDROID_SDK_ROOT%\licenses\google-gdk-license"

echo 33b6a2b64607f11b759f320ef9dff4ae5c47d97a > "%ANDROID_SDK_ROOT%\licenses\intel-android-extra-license"

echo 9a220f3e8e7c7f8b7f701e0b1e24a4b16ef0e2b5 > "%ANDROID_SDK_ROOT%\licenses\mips-android-sysimage-license"

echo All licenses accepted successfully!
