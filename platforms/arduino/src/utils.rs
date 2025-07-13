use anyhow::{Result, Context};
use log::{info, warn, error};
use std::path::PathBuf;
use std::fs;

/// Structure pour représenter un sketch Arduino
#[derive(Debug)]
pub struct ArduinoSketch {
    pub path: PathBuf,
    pub name: String,
    pub content: String,
}

impl ArduinoSketch {
    /// Charger un sketch depuis un fichier
    pub fn load_from_file(path: PathBuf) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .context("Failed to read sketch file")?;
        
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        Ok(ArduinoSketch {
            path,
            name,
            content,
        })
    }
    
    /// Valider le sketch (vérifications de base)
    pub fn validate(&self) -> Result<()> {
        info!("🔍 Validating sketch: {}", self.name);
        
        // Vérifier que le fichier a l'extension .ino
        if !self.path.extension().map_or(false, |ext| ext == "ino") {
            warn!("⚠️  File doesn't have .ino extension");
        }
        
        // Vérifier la présence des fonctions setup() et loop()
        let has_setup = self.content.contains("void setup()");
        let has_loop = self.content.contains("void loop()");
        
        if !has_setup {
            error!("❌ Missing setup() function");
            return Err(anyhow::anyhow!("Missing setup() function"));
        }
        
        if !has_loop {
            error!("❌ Missing loop() function");
            return Err(anyhow::anyhow!("Missing loop() function"));
        }
        
        println!("✅ Sketch validation passed");
        Ok(())
    }
}

/// Informations sur la carte Arduino
#[derive(Debug)]
pub struct BoardInfo {
    pub name: String,
    pub fqbn: String,
    pub description: String,
}

impl BoardInfo {
    /// Obtenir les informations de la carte par son nom
    pub fn get_board_info(board_name: &str) -> BoardInfo {
        match board_name {
            "uno" => BoardInfo {
                name: "Arduino Uno".to_string(),
                fqbn: "arduino:avr:uno".to_string(),
                description: "Arduino Uno R3 (ATmega328P)".to_string(),
            },
            "nano" => BoardInfo {
                name: "Arduino Nano".to_string(),
                fqbn: "arduino:avr:nano".to_string(),
                description: "Arduino Nano (ATmega328P)".to_string(),
            },
            "mega" => BoardInfo {
                name: "Arduino Mega".to_string(),
                fqbn: "arduino:avr:mega".to_string(),
                description: "Arduino Mega 2560 (ATmega2560)".to_string(),
            },
            "leonardo" => BoardInfo {
                name: "Arduino Leonardo".to_string(),
                fqbn: "arduino:avr:leonardo".to_string(),
                description: "Arduino Leonardo (ATmega32u4)".to_string(),
            },
            // ESP8266 boards
            "esp8266:esp8266:nodemcuv2" => BoardInfo {
                name: "NodeMCU 1.0 (ESP-12E Module)".to_string(),
                fqbn: "esp8266:esp8266:nodemcuv2".to_string(),
                description: "NodeMCU 1.0 (ESP-12E Module)".to_string(),
            },
            "esp8266:esp8266:nodemcu" => BoardInfo {
                name: "NodeMCU 0.9 (ESP-12 Module)".to_string(),
                fqbn: "esp8266:esp8266:nodemcu".to_string(),
                description: "NodeMCU 0.9 (ESP-12 Module)".to_string(),
            },
            "esp8266:esp8266:generic" => BoardInfo {
                name: "Generic ESP8266 Module".to_string(),
                fqbn: "esp8266:esp8266:generic".to_string(),
                description: "Generic ESP8266 Module".to_string(),
            },
            "esp8266:esp8266:d1_mini" => BoardInfo {
                name: "LOLIN(WEMOS) D1 R2 & mini".to_string(),
                fqbn: "esp8266:esp8266:d1_mini".to_string(),
                description: "LOLIN(WEMOS) D1 R2 & mini".to_string(),
            },
            // If it's a full FQBN, use it directly
            board_fqbn if board_fqbn.contains(':') => BoardInfo {
                name: board_fqbn.split(':').last().unwrap_or("Unknown").to_string(),
                fqbn: board_fqbn.to_string(),
                description: format!("Board: {}", board_fqbn),
            },
            _ => BoardInfo {
                name: "Arduino Uno (default)".to_string(),
                fqbn: "arduino:avr:uno".to_string(),
                description: "Arduino Uno R3 (ATmega328P) - Default".to_string(),
            },
        }
    }
    
    /// Lister toutes les cartes supportées
    pub fn list_supported_boards() -> Vec<BoardInfo> {
        vec![
            BoardInfo::get_board_info("uno"),
            BoardInfo::get_board_info("nano"),
            BoardInfo::get_board_info("mega"),
            BoardInfo::get_board_info("leonardo"),
        ]
    }
}

/// Vérifier si arduino-cli est installé
pub fn check_arduino_cli() -> Result<()> {
    info!("🔍 Checking arduino-cli installation...");
    
    let output = std::process::Command::new("arduino-cli")
        .args(&["version"])
        .output()
        .context("arduino-cli not found. Please install arduino-cli and add it to PATH")?;
    
    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        println!("✅ arduino-cli found: {}", version.trim());
        Ok(())
    } else {
        Err(anyhow::anyhow!("arduino-cli not working properly"))
    }
}

/// Créer un sketch d'exemple
pub fn create_example_sketch(path: PathBuf) -> Result<()> {
    // Si le path est juste un nom de fichier, créer la structure appropriée
    let (sketch_dir, sketch_file) = if path.extension().map_or(false, |ext| ext == "ino") {
        // Si c'est un fichier .ino, créer le dossier avec le même nom
        let file_stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("blink");
        let sketch_dir = PathBuf::from(file_stem);
        let sketch_file = sketch_dir.join(format!("{}.ino", file_stem));
        (sketch_dir, sketch_file)
    } else {
        // Si c'est un dossier, créer le fichier .ino avec le même nom
        let dir_name = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("blink");
        let sketch_file = path.join(format!("{}.ino", dir_name));
        (path, sketch_file)
    };
    
    // Créer le dossier s'il n'existe pas
    if !sketch_dir.exists() {
        fs::create_dir_all(&sketch_dir)?;
    }
    
    let example_content = r#"// Arduino LED Blink Example
// Simple example to test Arduino deployment

void setup() {
  // Initialize digital pin LED_BUILTIN as an output
  pinMode(LED_BUILTIN, OUTPUT);
  
  // Initialize serial communication
  Serial.begin(9600);
  Serial.println("Arduino LED Blink Example Started!");
}

void loop() {
  digitalWrite(LED_BUILTIN, HIGH);   // Turn the LED on
  Serial.println("LED ON");
  delay(1000);                       // Wait for a second
  
  digitalWrite(LED_BUILTIN, LOW);    // Turn the LED off
  Serial.println("LED OFF");
  delay(1000);                       // Wait for a second
}
"#;

    fs::write(&sketch_file, example_content)
        .context("Failed to create example sketch")?;
    
    println!("✅ Example sketch created at: {}", sketch_file.display());
    println!("📁 Sketch structure:");
    println!("  └── {}/", sketch_dir.display());
    println!("      └── {}", sketch_file.file_name().unwrap().to_str().unwrap());
    println!();
    println!("💡 Arduino CLI requires sketches to be in folders with the same name as the .ino file");
    
    Ok(())
}
