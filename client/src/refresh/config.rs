/// Configuration pour un rafraîchissement automatique
#[derive(Debug, Clone)]
pub struct RefreshConfig {
    /// Identifiant unique du rafraîchissement
    pub id: String,
    /// URL de l'endpoint à appeler
    pub endpoint: String,
    /// Intervalle de rafraîchissement en secondes
    pub interval_seconds: u32,
    /// Sélecteur CSS de l'élément à mettre à jour
    pub target_selector: String,
    /// Type de contenu à insérer
    pub content_type: ContentType,
    /// Champ JSON à extraire de la réponse (optionnel)
    pub json_field: Option<String>,
    /// Transformation à appliquer aux données
    pub transform: Option<DataTransform>,
    /// Afficher les erreurs dans l'interface
    pub show_errors: bool,    /// Sélecteurs de champs input dont les valeurs seront utilisées comme paramètres
    /// Format: (nom_parametre, selecteur_css)
    pub input_field_selectors: Vec<(String, String)>,
}

/// Type de contenu à insérer dans l'élément
#[derive(Debug, Clone)]
pub enum ContentType {
    /// Texte simple (textContent)
    Text,
    /// HTML (innerHTML)
    Html,
    /// Valeur d'un input (value)
    Value,
    /// Attribut spécifique
    Attribute(String),
}

/// Transformation des données avant affichage
#[derive(Debug, Clone)]
pub struct DataTransform {
    /// Préfixe à ajouter
    pub prefix: Option<String>,
    /// Suffixe à ajouter
    pub suffix: Option<String>,
    /// Format d'affichage (number, date, etc.)
    pub format: Option<String>,
}

impl RefreshConfig {    /// Constructeur simple pour un rafraîchissement de texte
    pub fn new_text(
        id: &str,
        endpoint: &str,
        interval_seconds: u32,
        target_selector: &str,
        json_field: Option<&str>,
    ) -> Self {
        Self {
            id: id.to_string(),
            endpoint: endpoint.to_string(),
            interval_seconds,
            target_selector: target_selector.to_string(),            content_type: ContentType::Text,
            json_field: json_field.map(|s| s.to_string()),
            transform: None,
            show_errors: true,
            input_field_selectors: Vec::new(),
        }
    }/// Constructeur pour un rafraîchissement HTML
    pub fn new_html(
        id: &str,
        endpoint: &str,
        interval_seconds: u32,
        target_selector: &str,
        json_field: Option<&str>,
    ) -> Self {
        Self {
            id: id.to_string(),
            endpoint: endpoint.to_string(),
            interval_seconds,
            target_selector: target_selector.to_string(),            content_type: ContentType::Html,
            json_field: json_field.map(|s| s.to_string()),
            transform: None,
            show_errors: true,
            input_field_selectors: Vec::new(),
        }
    }/// Ajouter une transformation
    pub fn with_transform(mut self, transform: DataTransform) -> Self {
        self.transform = Some(transform);
        self
    }    /// Ajouter un champ input comme source de paramètre
    pub fn with_input_field(mut self, param_name: &str, input_selector: &str) -> Self {
        self.input_field_selectors.push((param_name.to_string(), input_selector.to_string()));
        self
    }

    /// Désactiver l'affichage des erreurs
    pub fn without_errors(mut self) -> Self {
        self.show_errors = false;
        self
    }
}
