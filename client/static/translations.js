const translations = {
    fr: {
        login: "Identifiant",
        firstname: "Prénom",
        lastname: "Nom",
        age: "Âge",
        file: "Fichier",
        email: "Courriel",
        submit: "Envoyer",
        serverStatus: "État du Serveur",
        status: "Statut",
        message: "Message",
        checking: "Vérification...",
        connecting: "Connexion au serveur..."
    },
    en: {
        login: "Login",
        firstname: "Firstname",
        lastname: "Lastname",
        age: "Age",
        file: "File",
        email: "Email",
        submit: "Submit",
        serverStatus: "Server Status",
        status: "Status",
        message: "Message",
        checking: "Checking...",
        connecting: "Connecting to server..."
    }
};

export function translate(key, lang = 'fr') {
    return translations[lang][key] || key;
}
