#!/bin/bash
set -e

echo "[INFO] Configuration automatique ADB pour Android Docker..."

# Fonction pour installer ADB selon la distribution
install_adb() {
    echo "[INFO] Installation d'ADB..."
    
    if command -v apt-get &> /dev/null; then
        # Ubuntu/Debian
        sudo apt-get update
        sudo apt-get install -y android-tools-adb
    elif command -v yum &> /dev/null; then
        # RHEL/CentOS/Fedora
        sudo yum install -y android-tools
    elif command -v pacman &> /dev/null; then
        # Arch Linux
        sudo pacman -S android-tools
    elif command -v zypper &> /dev/null; then
        # openSUSE
        sudo zypper install android-tools
    else
        echo "[INFO] Installation manuelle des Android Platform Tools..."
        
        # Téléchargement manuel
        if [ ! -d "platform-tools" ]; then
            echo "[INFO] Téléchargement des Android Platform Tools..."
            wget -q https://dl.google.com/android/repository/platform-tools-latest-linux.zip -O platform-tools.zip
            unzip -q platform-tools.zip
            rm platform-tools.zip
        fi
        
        # Ajouter au PATH temporairement
        export PATH="$PWD/platform-tools:$PATH"
    fi
}

# Vérifier si ADB est installé
if ! command -v adb &> /dev/null; then
    echo "[ERROR] ADB non trouvé. Installation automatique..."
    install_adb
fi

echo "[INFO] Configuration des permissions USB..."
# Créer les règles udev pour Android si elles n'existent pas
if [ ! -f /etc/udev/rules.d/51-android.rules ]; then
    echo "[INFO] Création des règles udev pour Android..."
    sudo tee /etc/udev/rules.d/51-android.rules > /dev/null << 'EOF'
# Android devices udev rules
SUBSYSTEM=="usb", ATTR{idVendor}=="0bb4", MODE="0666", GROUP="plugdev" # HTC
SUBSYSTEM=="usb", ATTR{idVendor}=="0e79", MODE="0666", GROUP="plugdev" # Archos
SUBSYSTEM=="usb", ATTR{idVendor}=="0489", MODE="0666", GROUP="plugdev" # Foxconn
SUBSYSTEM=="usb", ATTR{idVendor}=="04c5", MODE="0666", GROUP="plugdev" # Fujitsu
SUBSYSTEM=="usb", ATTR{idVendor}=="04c5", MODE="0666", GROUP="plugdev" # Fujitsu-Toshiba
SUBSYSTEM=="usb", ATTR{idVendor}=="091e", MODE="0666", GROUP="plugdev" # Garmin-Asus
SUBSYSTEM=="usb", ATTR{idVendor}=="18d1", MODE="0666", GROUP="plugdev" # Google
SUBSYSTEM=="usb", ATTR{idVendor}=="201e", MODE="0666", GROUP="plugdev" # Haier
SUBSYSTEM=="usb", ATTR{idVendor}=="109b", MODE="0666", GROUP="plugdev" # Hisense
SUBSYSTEM=="usb", ATTR{idVendor}=="0bb4", MODE="0666", GROUP="plugdev" # HTC
SUBSYSTEM=="usb", ATTR{idVendor}=="12d1", MODE="0666", GROUP="plugdev" # Huawei
SUBSYSTEM=="usb", ATTR{idVendor}=="24e3", MODE="0666", GROUP="plugdev" # K-Touch
SUBSYSTEM=="usb", ATTR{idVendor}=="2116", MODE="0666", GROUP="plugdev" # KT Tech
SUBSYSTEM=="usb", ATTR{idVendor}=="0482", MODE="0666", GROUP="plugdev" # Kyocera
SUBSYSTEM=="usb", ATTR{idVendor}=="17ef", MODE="0666", GROUP="plugdev" # Lenovo
SUBSYSTEM=="usb", ATTR{idVendor}=="1004", MODE="0666", GROUP="plugdev" # LG
SUBSYSTEM=="usb", ATTR{idVendor}=="22b8", MODE="0666", GROUP="plugdev" # Motorola
SUBSYSTEM=="usb", ATTR{idVendor}=="0409", MODE="0666", GROUP="plugdev" # NEC
SUBSYSTEM=="usb", ATTR{idVendor}=="2080", MODE="0666", GROUP="plugdev" # Nook
SUBSYSTEM=="usb", ATTR{idVendor}=="0955", MODE="0666", GROUP="plugdev" # Nvidia
SUBSYSTEM=="usb", ATTR{idVendor}=="2257", MODE="0666", GROUP="plugdev" # OTGV
SUBSYSTEM=="usb", ATTR{idVendor}=="10a9", MODE="0666", GROUP="plugdev" # Pantech
SUBSYSTEM=="usb", ATTR{idVendor}=="1d4d", MODE="0666", GROUP="plugdev" # Pegatron
SUBSYSTEM=="usb", ATTR{idVendor}=="0471", MODE="0666", GROUP="plugdev" # Philips
SUBSYSTEM=="usb", ATTR{idVendor}=="04da", MODE="0666", GROUP="plugdev" # PMC-Sierra
SUBSYSTEM=="usb", ATTR{idVendor}=="05c6", MODE="0666", GROUP="plugdev" # Qualcomm
SUBSYSTEM=="usb", ATTR{idVendor}=="1f53", MODE="0666", GROUP="plugdev" # SK Telesys
SUBSYSTEM=="usb", ATTR{idVendor}=="04e8", MODE="0666", GROUP="plugdev" # Samsung
SUBSYSTEM=="usb", ATTR{idVendor}=="04dd", MODE="0666", GROUP="plugdev" # Sharp
SUBSYSTEM=="usb", ATTR{idVendor}=="054c", MODE="0666", GROUP="plugdev" # Sony
SUBSYSTEM=="usb", ATTR{idVendor}=="0fce", MODE="0666", GROUP="plugdev" # Sony Ericsson
SUBSYSTEM=="usb", ATTR{idVendor}=="2340", MODE="0666", GROUP="plugdev" # Teleepoch
SUBSYSTEM=="usb", ATTR{idVendor}=="0930", MODE="0666", GROUP="plugdev" # Toshiba
SUBSYSTEM=="usb", ATTR{idVendor}=="19d2", MODE="0666", GROUP="plugdev" # ZTE
EOF
    
    # Recharger les règles udev
    sudo udevadm control --reload-rules
    sudo udevadm trigger
fi

# Ajouter l'utilisateur au groupe plugdev si nécessaire
if ! groups | grep -q plugdev; then
    echo "[INFO] Ajout de l'utilisateur au groupe plugdev..."
    sudo usermod -a -G plugdev $USER
    echo "[WARNING] Vous devez vous déconnecter/reconnecter pour que les changements prennent effet."
    echo "          Ou utilisez: newgrp plugdev"
fi

echo "[INFO] Démarrage du serveur ADB sur Linux..."
adb kill-server
adb start-server

echo "[INFO] Vérification des appareils connectés..."
adb devices

# Fonction pour vérifier les appareils
check_device() {
    local device_count=$(adb devices | grep -c "device$" || true)
    return $((device_count == 0))
}

# Attendre que l'utilisateur connecte son téléphone
while check_device; do
    echo "[ATTENTION] Aucun appareil détecté !"
    echo "Assurez-vous que :"
    echo "1. Votre téléphone est branché en USB"
    echo "2. Le débogage USB est activé"
    echo "3. Vous avez autorisé le débogage sur votre téléphone"
    echo "4. Les règles udev sont appliquées (redémarrez si nécessaire)"
    echo ""
    echo "Appuyez sur Entrée pour réessayer ou Ctrl+C pour annuler..."
    read -r
    adb devices
done

echo "[OK] Appareil Android détecté !"
adb devices

echo "[INFO] Lancement du conteneur Docker avec détection automatique..."
echo "Mode : Détection automatique (USB direct sur Linux, TCP en fallback)"
echo

# Exporter les variables pour docker-compose
export ADB_MODE="auto"
export ADB_TCP_HOST=""

docker-compose up --build

if [ $? -ne 0 ]; then
    echo
    echo "[ERROR] Le build Docker a échoué !"
    echo
    echo "Diagnostic :"
    echo "1. Vérifiez que votre téléphone est toujours connecté : adb devices"
    echo "2. Redémarrez le serveur ADB : adb kill-server && adb start-server"
    echo "3. Vérifiez les permissions USB : ls -la /dev/bus/usb"
    echo "4. Relancez ce script"
    echo
    exit 1
fi

echo
echo "=== BUILD TERMINÉ AVEC SUCCÈS ==="
echo "L'APK a été généré, installé et l'application est lancée !"
echo "Les logs s'affichent en continu. Appuyez sur Ctrl+C pour arrêter."
