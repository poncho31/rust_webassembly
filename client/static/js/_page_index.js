import { translate } from '/js/translations.js';
import init from "/pkg/client.js";
// Traduire tous les éléments avec data-translate
document.querySelectorAll('[data-translate]').forEach(element => {
    const key = element.getAttribute('data-translate');
    element.textContent = translate(key);
});            // Gérer l'affichage du nom des fichiers choisis
function setupFileDisplay() {
    const fileInput = document.getElementById('files');
    const fileName = document.querySelector('.file-name');
    
    if (fileInput && fileName) {
        fileInput.addEventListener('change', function(event) {
            const files = event.target.files;
            if (files.length > 0) {
                if (files.length === 1) {
                    fileName.textContent = files[0].name;
                } else {
                    fileName.textContent = `${files.length} ${translate('files_selected')}`;
                }
                fileName.title = Array.from(files).map(f => f.name).join(', ');
            } else {
                fileName.textContent = translate('no_file_chosen');
                fileName.title = '';
            }
        });
    }
}            // Initialiser après le chargement du DOM
document.addEventListener('DOMContentLoaded', setupFileDisplay);            // Table form_data gérée entièrement par WebAssembly Rust
// (rafraîchissement automatique configuré dans lib.rs)
init();