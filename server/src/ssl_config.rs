use std::fs;
use std::path::Path;
use std::io::BufReader;
use rustls::ServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use rcgen::generate_simple_self_signed;

pub struct SslConfig;

impl SslConfig {
    /// Crée automatiquement un certificat SSL auto-signé si nécessaire
    pub fn create_ssl_acceptor() -> Result<ServerConfig, Box<dyn std::error::Error>> {
        let cert_dir = "certs";
        let cert_path = format!("{}/cert.pem", cert_dir);
        let key_path = format!("{}/key.pem", cert_dir);

        // Créer le dossier certs s'il n'existe pas
        if !Path::new(cert_dir).exists() {
            fs::create_dir_all(cert_dir)?;
            println!("📁 Dossier 'certs' créé");
        }

        // Vérifier si les certificats existent déjà
        if !Path::new(&cert_path).exists() || !Path::new(&key_path).exists() {
            println!("🔧 Génération automatique du certificat SSL...");
            Self::generate_self_signed_cert(&cert_path, &key_path)?;
            println!("✅ Certificat SSL généré automatiquement");
        } else {
            println!("📋 Certificats SSL existants trouvés");
        }        // Charger les certificats
        let cert_file = &mut BufReader::new(fs::File::open(&cert_path)?);
        let key_file = &mut BufReader::new(fs::File::open(&key_path)?);

        let cert_chain = certs(cert_file)
            .collect::<Result<Vec<_>, _>>()?;

        let mut keys = pkcs8_private_keys(key_file)
            .collect::<Result<Vec<_>, _>>()?;
        
        if keys.is_empty() {
            return Err("No PKCS8-encoded private key found.".into());
        }
        let key = keys.remove(0);

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, rustls::pki_types::PrivateKeyDer::Pkcs8(key))?;

        Ok(config)
    }    
    
    /// Génère un certificat auto-signé avec rcgen
    fn generate_self_signed_cert(cert_path: &str, key_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Génération du certificat avec rcgen
        let subject_alt_names = vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "0.0.0.0".to_string(),
        ];        let cert = generate_simple_self_signed(subject_alt_names)?;

        // Sauvegarde du certificat
        fs::write(cert_path, cert.serialize_pem()?)?;
        println!("💾 Certificat sauvegardé: {}", cert_path);

        // Sauvegarde de la clé privée
        fs::write(key_path, cert.serialize_private_key_pem())?;
        println!("🔑 Clé privée sauvegardée: {}", key_path);

        Ok(())
    }

    /// Affiche les informations du certificat
    pub fn print_ssl_info() {
        println!("🔒 ===== CONFIGURATION SSL =====");
        println!("📄 Certificat: certs/cert.pem");
        println!("🗝️  Clé privée: certs/key.pem");
        println!("⚠️  Certificat auto-signé (développement uniquement)");
        println!("🌐 Valide pour: localhost, 127.0.0.1, 0.0.0.0");
        println!("🛡️  Backend: RustLS (natif Rust)");
        println!("📅 Validité: 1 an");
        println!("================================");
    }

    /// Vérifie si SSL est configuré correctement
    pub fn is_ssl_ready() -> bool {
        Path::new("certs/cert.pem").exists() && Path::new("certs/key.pem").exists()
    }

    /// Force la régénération du certificat
    pub fn regenerate_certificate() -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 Régénération forcée du certificat SSL...");
        
        // Supprimer les anciens certificats s'ils existent
        if Path::new("certs/cert.pem").exists() {
            fs::remove_file("certs/cert.pem")?;
        }
        if Path::new("certs/key.pem").exists() {
            fs::remove_file("certs/key.pem")?;
        }

        // Créer de nouveaux certificats
        Self::generate_self_signed_cert("certs/cert.pem", "certs/key.pem")?;
        println!("✅ Nouveau certificat SSL généré");
        
        Ok(())
    }
}
