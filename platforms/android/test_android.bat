@echo off
echo === Testing Android APK ===

echo Checking ADB connection...
adb devices

echo Installing APK...
adb install -r app\build\outputs\apk\debug\app-debug.apk

echo Starting app...
adb shell am start -n com.webassembly.unified/.MainActivity

echo Monitoring logs (Ctrl+C to stop)...
adb logcat -c
adb logcat -s WebAssemblyApp