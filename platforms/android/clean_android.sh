#!/bin/bash
set -e

echo "=== NETTOYAGE ANDROID - DÉSINSTALLATION DE L'APK ==="
echo

cd "$(dirname "$0")"

# Vérification d'ADB
if ! command -v adb &> /dev/null; then
    echo "[ERROR] ADB non trouvé !"
    echo "Lancez d'abord ./build_android_docker.sh pour installer ADB"
    exit 1
fi

echo "Vérification des appareils connectés..."
adb devices

# Vérification du téléphone
if ! adb devices | grep -v "List of devices" | grep -q "device$"; then
    echo "[ERROR] Aucun téléphone détecté !"
    echo "Branchez votre téléphone et activez le débogage USB"
    exit 1
fi

echo
echo "Appareil détecté. Désinstallation de com.webassembly.unified..."
if adb uninstall com.webassembly.unified; then
    echo "✓ Application désinstallée avec succès !"
else
    echo "⚠ Application non trouvée ou déjà désinstallée"
fi

echo
echo "=== NETTOYAGE TERMINÉ ==="
echo "Vous pouvez maintenant relancer ./build_android_docker.sh"
