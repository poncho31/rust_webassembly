use std::fs;
use std::path::Path;
use actix_multipart::Field;
use futures::StreamExt;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn extract_file_info(field: &mut Field) -> String {
    let filename = field.content_disposition()
        .get_filename()
        .unwrap_or("unknown")
        .to_owned();
    
    let mut size = 0;
    while let Some(chunk) = field.next().await {
        if let Ok(data) = chunk {
            size += data.len();
        }
    }
    format!("{}({} bytes)", filename, size)
}

pub async fn extract_form_field(field: &mut Field) -> (String, String) {
    let name = field.content_disposition()
        .get_name()
        .unwrap_or("unknown")
        .to_owned();
    
    let mut value = String::new();
    while let Some(chunk) = field.next().await {
        if let Ok(data) = chunk {
            if let Ok(s) = String::from_utf8(data.to_vec()) {
                value.push_str(&s);
            }
        }
    }
    (name, value)
}

pub async fn save_uploaded_file(field: &mut Field, filename: &str) -> Result<String, std::io::Error> {
    // Créer le dossier Storage/form s'il n'existe pas
    let upload_dir = Path::new("storage/files");
    fs::create_dir_all(upload_dir)?;

    // Construire le chemin complet du fichier
    let file_path = upload_dir.join(filename);
    
    // Créer le fichier
    let mut file = File::create(&file_path).await?;
    
    // Lire le contenu du champ et écrire dans le fichier
    while let Some(chunk) = field.next().await {
        let data = chunk
            .map_err(|e| std::io::Error::new(
                std::io::ErrorKind::Other, 
                e.to_string()  // Convertir l'erreur en String pour être thread-safe
            ))?;
        file.write_all(&data).await?;
    }

    // Retourner la taille du fichier
    let metadata = file_path.metadata()?;
    Ok(format!("Saved {} bytes", metadata.len()))
}
