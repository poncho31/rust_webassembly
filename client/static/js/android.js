// Exécuter quand le DOM est prêt
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initAndroid);
} else {
    initAndroid();
}


function initAndroid() {
    // Vérification si Android est disponible
    if (window.Android) {
        console.log('✅ Android détecté');
        
        // Display if android
        Array.from(document.getElementsByClassName('is_android')).forEach(function(element) {
            element.classList.add('display_android');
            element.style.display = 'block'; // Force l'affichage
        });
 
        // Caméra et Photos
        document.getElementById('android_take_photo').addEventListener('click', function() {
            console.log('📸 Bouton photo cliqué');
            if (window.Android.takePhoto) {
                window.Android.takePhoto();
            } else {
                console.log('❌ Méthode takePhoto non disponible');
            }
        });

        document.getElementById('android_record_video').addEventListener('click', function() {
            console.log('🎥 Bouton vidéo cliqué');
            if (window.Android.recordVideo) {
                window.Android.recordVideo();
            } else {
                console.log('❌ Méthode recordVideo non disponible');
            }
        });

        document.getElementById('android_pick_image').addEventListener('click', function() {
            console.log('🖼️ Bouton galerie cliqué');
            if (window.Android.pickImage) {
                window.Android.pickImage();
            } else {
                console.log('❌ Méthode pickImage non disponible');
            }
        });

        // Fichiers et Stockage
        document.getElementById('android_pick_file').addEventListener('click', function() {
            console.log('📁 Bouton fichier cliqué');
            if (window.Android.pickFile) {
                window.Android.pickFile();
            } else {
                console.log('❌ Méthode pickFile non disponible');
            }
        });

        document.getElementById('android_save_file').addEventListener('click', function() {
            console.log('💾 Bouton sauvegarder cliqué');
            if (window.Android.saveFile) {
                window.Android.saveFile("test.txt", "Contenu de test");
            } else {
                console.log('❌ Méthode saveFile non disponible');
            }
        });

        // Localisation
        document.getElementById('android_get_location').addEventListener('click', function() {
            console.log('📍 Bouton position cliqué');
            if (window.Android.getLocation) {
                window.Android.getLocation();
            } else {
                console.log('❌ Méthode getLocation non disponible');
            }
        });

        document.getElementById('android_start_gps').addEventListener('click', function() {
            console.log('🗺️ Bouton GPS cliqué');
            if (window.Android.startGPS) {
                window.Android.startGPS();
            } else {
                console.log('❌ Méthode startGPS non disponible');
            }
        });

        // Capteurs
        document.getElementById('android_get_battery').addEventListener('click', function() {
            console.log('🔋 Bouton batterie cliqué');
            if (window.Android.getBatteryLevel) {
                const level = window.Android.getBatteryLevel();
                alert('Niveau de batterie: ' + level + '%');
            } else {
                console.log('❌ Méthode getBatteryLevel non disponible');
            }
        });

        document.getElementById('android_get_network').addEventListener('click', function() {
            console.log('📶 Bouton réseau cliqué');
            if (window.Android.getNetworkInfo) {
                const info = window.Android.getNetworkInfo();
                alert('État réseau: ' + info);
            } else {
                console.log('❌ Méthode getNetworkInfo non disponible');
            }
        });

        document.getElementById('android_get_device_info').addEventListener('click', function() {
            console.log('📱 Bouton info appareil cliqué');
            if (window.Android.getDeviceInfo) {
                const info = window.Android.getDeviceInfo();
                alert('Info appareil: ' + info);
            } else {
                console.log('❌ Méthode getDeviceInfo non disponible');
            }
        });

        // Audio
        document.getElementById('android_record_audio').addEventListener('click', function() {
            console.log('🎤 Bouton audio cliqué');
            if (window.Android.recordAudio) {
                window.Android.recordAudio();
            } else {
                console.log('❌ Méthode recordAudio non disponible');
            }
        });

        document.getElementById('android_play_sound').addEventListener('click', function() {
            console.log('🔊 Bouton son cliqué');
            if (window.Android.playSound) {
                window.Android.playSound("notification");
            } else {
                console.log('❌ Méthode playSound non disponible');
            }
        });

        // Vibration et Notifications
        document.getElementById('android_vibrate').addEventListener('click', function() {
            console.log('📳 Bouton vibration cliqué');
            if (window.Android.vibrate) {
                window.Android.vibrate(500); // 500ms
            } else {
                console.log('❌ Méthode vibrate non disponible');
            }
        });

        document.getElementById('android_show_notification').addEventListener('click', function() {
            console.log('🔔 Bouton notification cliqué');
            if (window.Android.showNotification) {
                window.Android.showNotification("Titre", "Message de notification");
            } else {
                console.log('❌ Méthode showNotification non disponible');
            }
        });

        document.getElementById('android_show_toast').addEventListener('click', function() {
            console.log('💬 Bouton toast cliqué');
            if (window.Android.showToast) {
                window.Android.showToast("Message toast de test");
            } else {
                console.log('❌ Méthode showToast non disponible');
            }
        });

        // Communication
        document.getElementById('android_send_sms').addEventListener('click', function() {
            console.log('� Bouton SMS cliqué');
            if (window.Android.sendSMS) {
                window.Android.sendSMS("", "Message de test");
            } else {
                console.log('❌ Méthode sendSMS non disponible');
            }
        });

        document.getElementById('android_make_call').addEventListener('click', function() {
            console.log('📞 Bouton appel cliqué');
            if (window.Android.makeCall) {
                window.Android.makeCall("");
            } else {
                console.log('❌ Méthode makeCall non disponible');
            }
        });

        document.getElementById('android_send_email').addEventListener('click', function() {
            console.log('✉️ Bouton email cliqué');
            if (window.Android.sendEmail) {
                window.Android.sendEmail("", "Sujet", "Corps du message");
            } else {
                console.log('❌ Méthode sendEmail non disponible');
            }
        });

        // Système
        document.getElementById('android_share_content').addEventListener('click', function() {
            console.log('🔗 Bouton partager cliqué');
            if (window.Android.shareContent) {
                window.Android.shareContent("Contenu à partager", "text/plain");
            } else {
                console.log('❌ Méthode shareContent non disponible');
            }
        });

        document.getElementById('android_open_browser').addEventListener('click', function() {
            console.log('🌐 Bouton navigateur cliqué');
            if (window.Android.openBrowser) {
                window.Android.openBrowser("https://www.google.com");
            } else {
                console.log('❌ Méthode openBrowser non disponible');
            }
        });

        document.getElementById('android_close_app').addEventListener('click', function() {
            console.log('❌ Bouton fermer app cliqué');
            if (window.Android.closeApp) {
                if (confirm('Voulez-vous vraiment fermer l\'application ?')) {
                    window.Android.closeApp();
                }
            } else {
                console.log('❌ Méthode closeApp non disponible');
            }
        });

    } else {
        console.log('❌ Android non détecté');
    }
}

