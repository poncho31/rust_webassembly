#!/bin/bash
set -e

source $CARGO_HOME/env

echo "=== CONFIGURATION ADB ==="

# Détection automatique de l'hôte Docker
detect_docker_host() {
    # Essayer plusieurs méthodes pour trouver l'hôte Docker
    local hosts=(
        "host.docker.internal"  # Docker Desktop (Windows/Mac)
        "172.17.0.1"           # Docker Linux par défaut
        "192.168.65.2"         # Docker Desktop sur certaines configs
        "10.0.2.2"             # Docker Toolbox
    )
    
    for host in "${hosts[@]}"; do
        echo "Test de connexion à $host:5037..."
        if timeout 2 bash -c "</dev/tcp/$host/5037"; then
            echo "✓ Serveur ADB trouvé sur $host:5037"
            echo "$host"
            return 0
        fi
    done
    
    echo "✗ Aucun serveur ADB trouvé"
    return 1
}

# Configuration selon le mode
if [ "$ADB_MODE" = "tcp" ]; then
    echo "==> Mode ADB TCP (utilisation du serveur ADB de l'hôte)"
    
    # Détection automatique de l'hôte
    if [ -z "$ADB_TCP_HOST" ]; then
        ADB_TCP_HOST=$(detect_docker_host)
        if [ $? -ne 0 ]; then
            echo "[ERROR] Impossible de trouver le serveur ADB de l'hôte !"
            echo ""
            echo "Solutions :"
            echo "1. Vérifiez que ADB fonctionne sur votre PC :"
            echo "   - Lancez 'adb devices' dans un terminal Windows"
            echo "   - Le serveur ADB doit être démarré"
            echo ""
            echo "2. Sur Windows, assurez-vous que Docker Desktop est configuré :"
            echo "   - Paramètres > Resources > Network"
            echo "   - Vérifiez que host.docker.internal est disponible"
            echo ""
            echo "3. Redémarrez le serveur ADB avec :"
            echo "   adb kill-server && adb start-server"
            exit 1
        fi
    fi
    
    echo "Connexion au serveur ADB sur $ADB_TCP_HOST:${ADB_TCP_PORT:-5037}"
    export ADB_SERVER_SOCKET=tcp:$ADB_TCP_HOST:${ADB_TCP_PORT:-5037}
    
    # Test de connexion
    echo "Test de la connexion ADB..."
    if ! timeout 5 adb devices >/dev/null 2>&1; then
        echo "[ERROR] Impossible de se connecter au serveur ADB !"
        echo "Serveur testé : $ADB_TCP_HOST:${ADB_TCP_PORT:-5037}"
        echo ""
        echo "Dépannage :"
        echo "1. Sur votre PC Windows, lancez : adb devices"
        echo "2. Vérifiez que votre téléphone apparaît"
        echo "3. Relancez ce conteneur"
        exit 1
    fi
    
else
    echo "==> Mode ADB USB (accès direct aux périphériques)"
    adb kill-server 2>/dev/null || true
    adb start-server
fi

echo "=== BUILD APK ==="

echo "Construction des bibliothèques natives..."
cargo ndk -t aarch64-linux-android -p 21 -- build --release
cargo ndk -t armv7-linux-androideabi -p 21 -- build --release
cargo ndk -t x86_64-linux-android -p 21 -- build --release
cargo ndk -t i686-linux-android -p 21 -- build --release

echo "Copie des bibliothèques natives..."
mkdir -p app/src/main/jniLibs/{arm64-v8a,armeabi-v7a,x86_64,x86}
cp /workspace/target/aarch64-linux-android/release/libwebassembly_android.so app/src/main/jniLibs/arm64-v8a/
cp /workspace/target/armv7-linux-androideabi/release/libwebassembly_android.so app/src/main/jniLibs/armeabi-v7a/
cp /workspace/target/x86_64-linux-android/release/libwebassembly_android.so app/src/main/jniLibs/x86_64/
cp /workspace/target/i686-linux-android/release/libwebassembly_android.so app/src/main/jniLibs/x86/

echo "Copie des ressources web..."
rm -rf app/src/main/assets/static
mkdir -p app/src/main/assets/static
cp -r /workspace/client/static/* app/src/main/assets/static/

echo "Construction de l'APK..."
sed -i 's/\r$//' gradlew
chmod +x gradlew
./gradlew assembleDebug
echo "✓ APK généré : app/build/outputs/apk/debug/app-debug.apk"

echo "=== DÉTECTION DU TÉLÉPHONE ==="

# Attente et détection des appareils
echo "Attente de la détection des appareils..."
sleep 3

echo "Appareils disponibles :"
adb devices

# Sélection du premier appareil physique (non émulateur)
DEVICE_ID=$(adb devices | grep -v "List of devices" | grep -v "emulator" | grep "device$" | head -1 | cut -f1)

if [ -z "$DEVICE_ID" ]; then
    echo ""
    echo "[ERROR] Aucun appareil Android trouvé !"
    echo ""
    echo "Vérifiez que :"
    echo "1. ✓ Votre téléphone est branché en USB"
    echo "2. ✓ Le débogage USB est activé dans les options développeur"
    echo "3. ✓ Vous avez autorisé le débogage sur votre téléphone"
    echo "4. ✓ Les pilotes USB sont installés sur votre PC"
    
    if [ "$ADB_MODE" = "tcp" ]; then
        echo "5. ✓ Le serveur ADB fonctionne sur votre PC (adb devices)"
        echo ""
        echo "Test depuis votre PC Windows :"
        echo "  > adb devices"
        echo "  > adb kill-server && adb start-server"
        echo "  > adb devices"
    fi
    
    echo ""
    echo "État actuel des appareils ADB :"
    adb devices -l
    exit 1
fi

echo "✓ Utilisation de l'appareil : $DEVICE_ID"

echo "=== INSTALLATION DE L'APK ==="

# Vérifier si l'application est déjà installée
echo "Vérification de l'installation existante..."
if adb -s "$DEVICE_ID" shell pm list packages | grep -q "com.main"; then
    echo "⚠ Application déjà installée, désinstallation de l'ancienne version..."
    if adb -s "$DEVICE_ID" uninstall com.main; then
        echo "✓ Ancienne version désinstallée"
    else
        echo "⚠ Impossible de désinstaller l'ancienne version (peut-être déjà supprimée)"
    fi
else
    echo "✓ Aucune installation existante détectée"
fi

echo "Installation de l'APK sur $DEVICE_ID..."
if adb -s "$DEVICE_ID" install app/build/outputs/apk/debug/app-debug.apk; then
    echo "✓ APK installé avec succès"
else
    echo "[ERROR] Échec de l'installation de l'APK"
    echo ""
    echo "Solutions possibles :"
    echo "1. Vérifiez l'espace disque disponible sur le téléphone"
    echo "2. Vérifiez que l'installation d'apps inconnues est autorisée"
    echo "3. Redémarrez le téléphone et relancez le script"
    echo ""
    echo "Détails de l'erreur :"
    adb -s "$DEVICE_ID" install app/build/outputs/apk/debug/app-debug.apk
    exit 1
fi

echo "Lancement de l'application..."
adb -s "$DEVICE_ID" shell am start -n com.main/.MainActivity

echo "=== LOGS DE L'APPLICATION ==="
echo "Nettoyage des logs précédents..."
adb -s "$DEVICE_ID" logcat -c

echo "Affichage des logs (Ctrl+C pour arrêter) :"
echo "Tag de filtre : rust_webassembly_android"
echo ""

# Logs avec fallback si le tag spécifique n'existe pas
adb -s "$DEVICE_ID" logcat -s rust_webassembly_android | while read line; do
    echo "[$(date '+%H:%M:%S')] $line"
done
