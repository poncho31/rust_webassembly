use serde_json::Value;

pub struct Table<'a> {
    data: &'a Value,
    css_class: String,
}

impl<'a> Table<'a> {
    /// Crée une table HTML à partir de données JSON
    pub fn create(data: &'a Value, css_class: &str) -> Self {
        Self {
            data,
            css_class : css_class.to_string(),
        }
    }

    pub fn with_css_class(mut self, css_class: &str) -> Self {
        self.css_class = css_class.to_string();
        self    
    }
      
    /// Génère le code HTML de la table en fonction du type de données JSON
    /// Crée une table simple clé-valeur avec tables imbriquées pour les tableaux
    pub fn to_html(&self) -> String {
        match self.data {
            Value::Object(map) 
                => self.generate_table(map),
                _ => format!(
                    "<table class='{}'><tr><td>Simple Value</td><td>{}</td></tr></table>", 
                    self.css_class, 
                    html_escape::encode_text(&self.data.to_string())
                )
        }
    }    
    
    /// Génère une table avec les clés comme en-têtes de colonnes
    fn generate_table(&self, map: &serde_json::Map<String, Value>) -> String {
        // Créer les en-têtes à partir des clés
        let headers: Vec<String> = map.keys()
            .map(|key| format!("<th>{}</th>", html_escape::encode_text(key)))
            .collect();

        // Créer une seule rangée avec toutes les valeurs
        let cells: Vec<String> = map.values()
            .map(|value| {
                let value_html = self.format_value(value);
                format!("<td>{}</td>", value_html)
            })
            .collect();

        format!(
            "<table class='{}'><thead><tr>{}</tr></thead><tbody><tr>{}</tr></tbody></table>",
            self.css_class,
            headers.join(""),
            cells.join("")
        )
    }
    
    /// Formate une valeur - récursion pour objets et tableaux
    fn format_value(&self, value: &Value) -> String {
        match value {
            Value::Object(obj) => {
                if obj.is_empty() {
                    "Empty object".to_string()
                } else {
                    // Récursion : réutilise generate_table pour les objets imbriqués
                    format!("<span class='nested-table'>{}</span>", self.generate_table(obj))
                }
            }
            Value::Array(arr) => {
                if arr.is_empty() {
                    "Empty array".to_string()
                } else {
                    self.create_array_table(arr)
                }
            }
            _ => html_escape::encode_text(&value.to_string()).to_string()
        }
    }

    /// Crée une table pour un tableau avec récursion pour les éléments complexes
    fn create_array_table(&self, arr: &Vec<Value>) -> String {
        let rows: Vec<String> = arr.iter()
            .enumerate()
            .map(|(i, item)| {
                let item_html = match item {
                    Value::Object(obj) => {
                        if obj.is_empty() {
                            "Empty object".to_string()
                        } else {
                            // Récursion : réutilise generate_table pour les objets dans le tableau
                            format!("<div class='nested-table'>{}</div>", self.generate_table(obj))
                        }
                    }
                    _ => self.format_value(item) // Récursion pour autres types
                };
                
                format!("<tr><td>{}</td><td>{}</td></tr>", i + 1, item_html)
            })
            .collect();

        format!(
            "<table class='sub-table'><tbody>{}</tbody></table>",
            rows.join("")
        )
    }
}
