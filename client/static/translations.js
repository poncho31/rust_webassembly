const translations = {
    fr: {
        login: "Identifiant",
        birthday : "Date de naissance",
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
        birthday: "Birthday",
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
