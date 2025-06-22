function initAndroid() {
    // Vérification si Android est disponible
    if (window.Android) {
        console.log('✅ Android détecté');
        
        // Display if android
        Array.from(document.getElementsByClassName('is_android')).forEach(function(element) {
            element.classList.add('display_android');
            element.style.display = 'block'; // Force l'affichage
        });
 
        // Take photo
        document.getElementById('android_take_photo').addEventListener('click', function() {
            console.log('📸 Bouton photo cliqué');
            window.Android.takePhoto();
        });

    } else {
        console.log('❌ Android non détecté');
    }
}

// Exécuter quand le DOM est prêt
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initAndroid);
} else {
    initAndroid();
}