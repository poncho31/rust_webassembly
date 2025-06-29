use anyhow::{Error, Result};
use uuid::Uuid;
use time::OffsetDateTime;
use crate::{repositories::_database::DatabaseQuery};
use std::collections::HashMap;
use std::fs;

pub struct InitRepository {
    db: DatabaseQuery,
}

impl InitRepository {
    pub fn new(db_query: DatabaseQuery) -> Self {
        Self { db: db_query }
    }

    // Méthode pour initialiser un repository à partir d'une structure de table
    pub async fn init_repository(&self, repository_name: &str, table_schema: &str) -> Result<()> {
        println!("Initializing repository: {}", repository_name);
        
        // Analyser le schéma de la table pour extraire les colonnes et les types
        let columns = self.parse_table_schema(table_schema)?;
        
        // Générer le code du repository
        let repository_code = self.generate_repository_code(repository_name, &columns)?;
        
        // Définir le chemin du fichier
        let file_path = format!("core/src/repositories/{}_repository.rs", repository_name);
        
        // vérifier que le fichier n'existe pas déjà
        if !fs::metadata(&file_path).is_ok() {
            // Écrire le code dans un fichier
            std::fs::write(&file_path, repository_code)
                .map_err(|e| Error::msg(format!("Failed to write repository file: {}", e)))?;
            
            // Mettre à jour le fichier mod.rs pour inclure le nouveau module
            self.update_mod_rs(repository_name)?;
            
            println!("\x1b[32mRepository {}initialized successfully\x1b[0m", repository_name);        }
        else{
            println!("\x1b[33mRepository {} already exist\x1b[0m", repository_name);
        }

        Ok(())
    }
    
    // Analyse le schéma d'une table pour en extraire les colonnes et leurs types
    fn parse_table_schema(&self, table_schema: &str) -> Result<Vec<(String, String, bool)>> {
        let mut columns = Vec::new();
        
        // Diviser le schéma en lignes et analyser chaque colonne
        for line in table_schema.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("--") {
                continue;
            }
            
            // Diviser la ligne en parties (nom, type, contraintes)
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            
            let col_name = parts[0].trim().to_string();
            let mut col_type = parts[1].trim().to_string();
            
            // Déterminer si la colonne est nullable
            let is_nullable = !line.contains("NOT NULL");
            
            // Mapper les types SQL vers les types Rust
            col_type = match col_type.to_uppercase().as_str() {
                "UUID" => "Uuid".to_string(),
                "TEXT" | "VARCHAR" | "CHAR" => "String".to_string(),
                "INT" | "INTEGER" | "SMALLINT" => "i32".to_string(),
                "BIGINT" => "i64".to_string(),
                "BOOLEAN" | "BOOL" => "bool".to_string(),
                "FLOAT" | "REAL" => "f32".to_string(),
                "DOUBLE" | "NUMERIC" => "f64".to_string(),
                "TIMESTAMPTZ" | "TIMESTAMP" => "OffsetDateTime".to_string(),
                _ => "String".to_string(), // Type par défaut
            };
            
            columns.push((col_name, col_type, is_nullable));
        }
        
        Ok(columns)
    }    // Génère le code du repository
    fn generate_repository_code(&self, table_name: &str, columns: &[(String, String, bool)]) -> Result<String> {
        // Nom de la struct (première lettre en majuscule)
        let struct_name = table_name.chars().next().unwrap().to_uppercase().to_string() 
            + &table_name[1..].to_string();
        
        // En-tête du fichier        // Initialiser le code du repository
        let mut code = String::new();
        
        // En-tête du fichier avec imports
        code.push_str("use serde::{Serialize, Deserialize};\n");
        code.push_str("use anyhow::Result;\n");
        code.push_str("use time::OffsetDateTime;\n");
        code.push_str("use uuid::Uuid;\n");
        code.push_str("use crate::repositories::_database::DatabaseQuery;\n\n");
        
        // Définir la structure principale
        code.push_str("#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]\n");
        code.push_str(&format!("pub struct {} {{\n", struct_name));
        
        // Ajouter les champs de la structure
        for (name, type_name, is_nullable) in columns {
            let field_name = if name == "type" {
                "r#type".to_string()  // Gestion du mot-clé réservé
            } else {
                name.to_string()
            };
            
            let field_type = if *is_nullable {
                format!("Option<{}>", type_name)
            } else {
                type_name.to_string()
            };
            
            code.push_str(&format!("    pub {}: {},\n", field_name, field_type));
        }
        
        // Fermer la définition de la structure
        code.push_str("}\n\n");        // Ajouter la structure du repository
        code.push_str(&format!("pub struct {}Repository {{\n", struct_name));
        code.push_str("    pool: PgPool,\n");
        code.push_str("}\n\n");
        
        // Implémenter les méthodes sur la structure principale
        code.push_str(&format!("impl {} {{\n", struct_name));
        code.push_str("    pub fn new(");
        
        // Ajouter les paramètres du constructeur
        let mut constructor_params = Vec::new();
        let mut constructor_assignments = Vec::new();
        
        for (name, type_name, is_nullable) in columns {
            if name != "id" && name != "created_at" && name != "updated_at" {
                let field_name = if name == "type" {
                    "r#type".to_string()
                } else {
                    name.to_string()
                };
                
                let param_type = if *is_nullable {
                    format!("Option<&{}>", match type_name.as_str() {
                        "String" => "str",
                        _ => type_name,
                    })
                } else {
                    format!("&{}", match type_name.as_str() {
                        "String" => "str",
                        _ => type_name,
                    })
                };
                
                constructor_params.push(format!("{}: {}", field_name, param_type));
                
                if *is_nullable {
                    constructor_assignments.push(format!("        {}: {}.map(|s| s.to_string()),", 
                        field_name, field_name));
                } else if type_name == "String" {
                    constructor_assignments.push(format!("        {}: {}.to_string(),", field_name, field_name));
                } else {
                    constructor_assignments.push(format!("        {}: *{},", field_name, field_name));
                }
            }
        }
        
        code.push_str(&constructor_params.join(",\n        "));
        code.push_str(") -> Self {\n");
        code.push_str("        let now = OffsetDateTime::now_utc();\n");
        code.push_str("        Self {\n");
        code.push_str("            id: Uuid::new_v4(),\n");
        
        for assignment in constructor_assignments {
            code.push_str(&format!("            {}\n", assignment));
        }
        
        code.push_str("            created_at: now,\n");
        code.push_str("            updated_at: now,\n");
        code.push_str("        }\n");
        code.push_str("    }\n");
        code.push_str("}\n\n");
        // Ajouter la structure du Repository
        code.push_str(&format!("pub struct {}Repository {{\n", struct_name));
        code.push_str("    db: DatabaseQuery,\n");
        code.push_str("}\n\n");
        
        // Implémenter le repository
        code.push_str(&format!("impl {}Repository {{\n", struct_name));
        code.push_str("    pub fn new(pool: sqlx::PgPool) -> Self {\n");
        code.push_str("        Self { db: DatabaseQuery::new(pool) }\n");
        code.push_str("    }\n\n");
        code.push_str("    /// Crée un nouvel enregistrement\n");
        // Ajouter les noms de colonnes pour l'insertion
        let mut column_names = Vec::new();
        let mut placeholders = Vec::new();
        let mut binds = Vec::new();
        
        for (i, (name, _, _)) in columns.iter().enumerate().skip(1) {
            if name == "type" {
                column_names.push("\\\"type\\\"".to_string());
            } else {
                column_names.push(name.to_string());
            }
            placeholders.push(format!("${}", i + 1));
            binds.push(format!("        .bind(&item.{})", if name == "type" { "r#type" } else { name }));
        }
        
        let column_list = column_names.join(", ");
        let placeholder_list = placeholders.join(", ");
        
        code.push_str(&format!("    pub async fn create(&self, item: &{}) -> Result<{}> {{\n", struct_name, struct_name));
        code.push_str(&format!("        let result = sqlx::query_as::<_, {}>(\n", struct_name));
        code.push_str(&format!("            r#\"INSERT INTO {} (id, {}) VALUES ($1, {}) RETURNING *\"#\n", 
            table_name,
            column_list,
            placeholder_list));
        code.push_str("        )\n");
        code.push_str("        .bind(&item.id)\n");
        
        for bind in binds {
            code.push_str(&format!("{}\n", bind));
        }
        
        code.push_str("        .fetch_one(&self.pool)\n");
        code.push_str("        .await?;\n\n");
        code.push_str("        Ok(result)\n");
        code.push_str("    }\n\n");
          // Implémenter find_by_id
        code.push_str("    /// Recherche un enregistrement par ID\n");
        code.push_str(&format!("    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<{}>> {{\n", struct_name));
        code.push_str(&format!("        let result = sqlx::query_as::<_, {}>(\n", struct_name));
        code.push_str(&format!("            r#\"SELECT * FROM {} WHERE id = $1\"#\n", table_name));
        code.push_str("        )\n");
        code.push_str("        .bind(id)\n");
        code.push_str("        .fetch_optional(&self.pool)\n");
        code.push_str("        .await?;\n\n");
        code.push_str("        Ok(result)\n");
        code.push_str("    }\n\n");        code.push_str("    /// Récupère tous les enregistrements\n");
        code.push_str(&format!("    pub async fn find_all(&self) -> Result<Vec<{}>> {{\n", struct_name));
        code.push_str(&format!("        let results = sqlx::query_as::<_, {}>(\n", struct_name));
        code.push_str(&format!("            r#\"SELECT * FROM {} ORDER BY created_at DESC\"#\n", table_name));
        code.push_str("        )\n");
        code.push_str("        .fetch_all(&self.pool)\n");
        code.push_str("        .await?;\n\n");
        code.push_str("        Ok(results)\n");
        code.push_str("    }\n\n");
          code.push_str("    /// Met à jour un enregistrement\n");
        
        // Ajouter les champs à mettre à jour
        let mut updates = Vec::new();
        let mut binds = Vec::new();
        
        for (i, (name, _, _)) in columns.iter().enumerate().skip(1) {
            if name != "id" && name != "created_at" {
                if name == "type" {
                    updates.push(format!("\\\"type\\\" = ${}", i + 1));
                } else {
                    updates.push(format!("{} = ${}", name, i + 1));
                }
                
                binds.push(format!("        .bind(&item.{})", if name == "type" { "r#type" } else { name }));
            }
        }
        
        updates.push(format!("updated_at = ${}", columns.len() + 1));
        
        code.push_str(&format!("    pub async fn update(&self, item: &{}) -> Result<{}> {{\n", struct_name, struct_name));
        code.push_str("        let now = OffsetDateTime::now_utc();\n\n");
        code.push_str(&format!("        let result = sqlx::query_as::<_, {}>(\n", struct_name));
        code.push_str(&format!("            r#\"UPDATE {} SET {}\"#\n", 
            table_name, 
            &updates.join(", ")));
        code.push_str("            .to_string() + \" WHERE id = $1 RETURNING *\"\n");
        code.push_str("        )\n");
        code.push_str("        .bind(&item.id)\n");
        
        for bind in binds {
            code.push_str(&format!("{}\n", bind));
        }
        
        code.push_str("        .bind(&now)\n");
        code.push_str("        .fetch_one(&self.pool)\n");
        code.push_str("        .await?;\n\n");
        code.push_str("        Ok(result)\n");
        code.push_str("    }\n\n");
          // Implémenter delete
        code.push_str("    /// Supprime un enregistrement\n");
        code.push_str("    pub async fn delete(&self, id: &Uuid) -> Result<bool> {\n");
        code.push_str("        let result = sqlx::query(\n");
        code.push_str(&format!("            r#\"DELETE FROM {} WHERE id = $1\"#\n", table_name));
        code.push_str("        )\n");
        code.push_str("        .bind(id)\n");
        code.push_str("        .execute(&self.pool)\n");
        code.push_str("        .await?;\n\n");
        code.push_str("        Ok(result.rows_affected() > 0)\n");
        code.push_str("    }\n");
        code.push_str("}");
        
        Ok(code)
    }
    
    // Met à jour le fichier mod.rs pour inclure le nouveau module
    fn update_mod_rs(&self, repository_name: &str) -> Result<()> {
        let mod_path = "src/repositories/mod.rs";
        
        // Lire le contenu actuel du fichier
        let content = match std::fs::read_to_string(&mod_path) {
            Ok(content) => content,
            Err(_) => String::new(), // Si le fichier n'existe pas, on crée un nouveau
        };
        
        // Vérifier si le module est déjà déclaré
        let module_line = format!("pub mod {}_repository;", repository_name);
        if content.contains(&module_line) {
            return Ok(());
        }
        
        // Ajouter la ligne pour le nouveau module
        let new_content = if content.is_empty() {
            module_line
        } else {
            format!("{}\n{}", content.trim_end(), module_line)
        };
        
        // Écrire le contenu mis à jour
        std::fs::write(mod_path, new_content)
            .map_err(|e| Error::msg(format!("Failed to update mod.rs: {}", e)))?;
        
        Ok(())
    }

}
