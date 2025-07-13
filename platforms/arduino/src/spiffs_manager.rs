//! SPIFFS Manager for ESP8266
//! 
//! This module handles SPIFFS filesystem setup and management for ESP8266 devices
//! Previously handled by setup_spiffs.bat

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use log::{warn, error};

pub struct SpiffsManager {
    pub verbose: bool,
    pub work_dir: PathBuf,
}

impl SpiffsManager {
    pub fn new(work_dir: PathBuf, verbose: bool) -> Self {
        SpiffsManager {
            verbose,
            work_dir,
        }
    }

    /// Setup SPIFFS data folder and prepare files for upload
    pub fn setup_spiffs_data(&self) -> Result<()> {
        if self.verbose {
            println!("📁 Setting up SPIFFS automatically for ESP8266...");
        }

        let html_path = self.work_dir.join("static").join("arduino.html");
        let config_path = self.work_dir.join("static").join("wifi_config.json");
        let data_dir = self.work_dir.join("static").join("data");
        let data_html_path = data_dir.join("arduino.html");
        let data_config_path = data_dir.join("wifi_config.json");

        // Check if arduino.html exists
        if !html_path.exists() {
            warn!("⚠️  arduino.html not found at: {}", html_path.display());
            warn!("⚠️  Using fallback HTML instead");
            return self.create_fallback_html(&data_dir);
        }

        if self.verbose {
            println!("✅ arduino.html found at: {}", html_path.display());
        }

        // Check if wifi_config.json exists
        if !config_path.exists() {
            if self.verbose {
                println!("⚠️  wifi_config.json not found, creating default config");
            }
            self.create_default_config(&config_path)?;
        } else if self.verbose {
            println!("✅ wifi_config.json found at: {}", config_path.display());
        }

        // Create data folder if it doesn't exist
        if !data_dir.exists() {
            if self.verbose {
                println!("📁 Creating data folder at: {}", data_dir.display());
            }
            fs::create_dir_all(&data_dir)
                .context("Failed to create data directory")?;
            if self.verbose {
                println!("✅ data folder created");
            }
        } else if self.verbose {
            println!("✅ data folder already exists");
        }

        // Copy HTML file to data folder
        if self.verbose {
            println!("📄 Copying arduino.html to data folder...");
        }
        
        fs::copy(&html_path, &data_html_path)
            .with_context(|| format!("Failed to copy {} to {}", html_path.display(), data_html_path.display()))?;

        // Copy config file to data folder
        if self.verbose {
            println!("📄 Copying wifi_config.json to data folder...");
        }
        
        fs::copy(&config_path, &data_config_path)
            .with_context(|| format!("Failed to copy {} to {}", config_path.display(), data_config_path.display()))?;

        if data_html_path.exists() && data_config_path.exists() {
            if self.verbose {
                println!("✅ Files prepared for SPIFFS:");
                
                // Display file sizes
                let html_metadata = fs::metadata(&data_html_path)?;
                let config_metadata = fs::metadata(&data_config_path)?;
                println!("   � arduino.html: {} bytes", html_metadata.len());
                println!("   ⚙️  wifi_config.json: {} bytes", config_metadata.len());
                
                if html_metadata.len() > 64 * 1024 {
                    warn!("⚠️  HTML file size is large ({} bytes) - consider minifying for better performance", html_metadata.len());
                }
            }
            
            self.display_spiffs_instructions()?;
        } else {
            error!("⚠️  Could not copy all files - using fallback");
            return self.create_fallback_html(&data_dir);
        }

        Ok(())
    }

    /// Create a default configuration file
    fn create_default_config(&self, config_path: &Path) -> Result<()> {
        let default_config = r#"{
  "wifi": {
    "ssid": "YOUR_WIFI_SSID",
    "password": "YOUR_WIFI_PASSWORD"
  },
  "device": {
    "name": "ESP8266-Complete",
    "version": "1.0.0",
    "port": 80
  },
  "sensors": {
    "interval": 5000,
    "auto_relay": false
  },
  "pins": {
    "led": "LED_BUILTIN",
    "relay": 12,
    "button": 0,
    "sensor": 14,
    "pwm": 13,
    "analog": "A0"
  }
}"#;

        fs::write(config_path, default_config)
            .context("Failed to create default configuration file")?;

        if self.verbose {
            println!("✅ Default wifi_config.json created");
            println!("⚠️  Please edit wifi_config.json with your WiFi credentials");
        }

        Ok(())
    }

    /// Create a fallback HTML file if the main one is missing
    fn create_fallback_html(&self, data_dir: &Path) -> Result<()> {
        if !data_dir.exists() {
            fs::create_dir_all(data_dir)?;
        }

        let fallback_html = r#"<!DOCTYPE html>
<html>
<head>
    <title>ESP8266 Fallback Server</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; background: #f0f0f0; }
        .container { max-width: 800px; margin: 0 auto; background: white; padding: 20px; border-radius: 8px; }
        h1 { color: #333; text-align: center; }
        .status { padding: 10px; margin: 10px 0; background: #e8f5e8; border-radius: 4px; }
        .btn { display: inline-block; padding: 10px 20px; margin: 5px; background: #007bff; color: white; text-decoration: none; border-radius: 4px; }
        .btn:hover { background: #0056b3; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🌐 ESP8266 Server</h1>
        <div class="status">
            <strong>Status:</strong> <span id="status">Loading...</span><br>
            <strong>Uptime:</strong> <span id="uptime">Loading...</span><br>
            <strong>Free Heap:</strong> <span id="heap">Loading...</span>
        </div>
        <div>
            <a href="/api/status" class="btn">Status API</a>
            <a href="/api/system" class="btn">System Info</a>
            <a href="/led/toggle" class="btn">Toggle LED</a>
        </div>
    </div>
    <script>
        function updateStatus() {
            fetch('/api/status')
                .then(response => response.json())
                .then(data => {
                    document.getElementById('status').textContent = 'Online';
                    document.getElementById('uptime').textContent = data.uptime + 's';
                    document.getElementById('heap').textContent = data.free_heap + ' bytes';
                })
                .catch(error => {
                    document.getElementById('status').textContent = 'Error';
                });
        }
        setInterval(updateStatus, 5000);
        updateStatus();
    </script>
</body>
</html>"#;

        let fallback_path = data_dir.join("arduino.html");
        fs::write(&fallback_path, fallback_html)
            .context("Failed to create fallback HTML file")?;

        if self.verbose {
            println!("✅ Fallback HTML created");
        }

        Ok(())
    }

    /// Display SPIFFS upload instructions
    fn display_spiffs_instructions(&self) -> Result<()> {
        if self.verbose {
            println!();
            println!("🎯 SPIFFS Setup Complete!");
            println!("══════════════════════════════════════════════");
            println!("📋 Next steps to enable full web interface:");
            println!();
            println!("1. 📁 Verify data folder structure:");
            println!("   static/");
            println!("   ├── esp8266_webserver.ino");
            println!("   ├── arduino.html");
            println!("   └── data/");
            println!("       └── arduino.html  ← This file is ready!");
            println!();
            println!("2. 📡 Upload to ESP8266:");
            println!("   • Arduino IDE: Tools → ESP8266 Sketch Data Upload");
            println!("   • PlatformIO: pio run -t uploadfs");
            println!();
            println!("3. 🚀 Deploy the sketch:");
            println!("   • Upload the .ino file to your ESP8266");
            println!("   • Monitor serial output for IP address");
            println!();
            println!("4. 🌐 Access the web interface:");
            println!("   • Open http://[ESP8266-IP] in your browser");
            println!("   • You should see the full HTML interface");
            println!();
            println!("💡 Use: cargo run -- setup-spiffs");
            println!("   to run this setup again anytime");
            println!("══════════════════════════════════════════════");
        }
        Ok(())
    }

    /// Validate SPIFFS setup and provide troubleshooting
    pub fn validate_spiffs_setup(&self) -> Result<bool> {
        let data_dir = self.work_dir.join("static").join("data");
        let html_file = data_dir.join("arduino.html");
        
        if !data_dir.exists() {
            if self.verbose {
                println!("❌ SPIFFS validation failed: data folder not found");
                println!("💡 Run: cargo run -- setup-spiffs");
            }
            return Ok(false);
        }
        
        if !html_file.exists() {
            if self.verbose {
                println!("❌ SPIFFS validation failed: arduino.html not found in data folder");
                println!("💡 Run: cargo run -- setup-spiffs");
            }
            return Ok(false);
        }
        
        let metadata = fs::metadata(&html_file)?;
        if metadata.len() == 0 {
            if self.verbose {
                println!("❌ SPIFFS validation failed: arduino.html is empty");
                println!("💡 Run: cargo run -- setup-spiffs");
            }
            return Ok(false);
        }
        
        if self.verbose {
            println!("✅ SPIFFS data validated successfully");
            println!("   📁 Data folder: {}", data_dir.display());
            println!("   📄 HTML file: {} bytes", metadata.len());
        }
        
        Ok(true)
    }

    /// Auto-setup SPIFFS if not already configured
    pub fn auto_setup_if_needed(&self) -> Result<()> {
        if !self.validate_spiffs_setup()? {
            if self.verbose {
                println!("🔄 SPIFFS not configured, setting up automatically...");
            }
            self.setup_spiffs_data()?;
        } else if self.verbose {
            println!("✅ SPIFFS already configured");
        }
        Ok(())
    }

    /// Get SPIFFS upload command for different tools
    pub fn get_spiffs_upload_commands(&self) -> Vec<String> {
        vec![
            "Arduino IDE: Tools > ESP8266 Sketch Data Upload".to_string(),
            "PlatformIO: pio run -t uploadfs".to_string(),
            "ESP Tool: esptool.py --port COM_PORT write_flash 0x300000 spiffs.bin".to_string(),
        ]
    }

    /// Generate SPIFFS troubleshooting info
    pub fn get_troubleshooting_info(&self) -> Vec<String> {
        vec![
            "If SPIFFS upload fails:".to_string(),
            "- Make sure correct board is selected in Arduino IDE".to_string(),
            "- Check that COM port is available".to_string(),
            "- Verify ESP8266 is not running (press and hold BOOT button)".to_string(),
            "- Try lower upload speed (115200 or 57600)".to_string(),
            "".to_string(),
            "If file is not found after upload:".to_string(),
            "- Check Serial Monitor for SPIFFS messages".to_string(),
            "- Verify file path is /arduino.html (not /data/arduino.html)".to_string(),
            "- Try reformatting SPIFFS: SPIFFS.format()".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_spiffs_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let manager = SpiffsManager::new(temp_dir.path().to_path_buf(), false);
        assert_eq!(manager.work_dir, temp_dir.path().to_path_buf());
    }

    #[test]
    fn test_validate_spiffs_setup_empty() {
        let temp_dir = tempdir().unwrap();
        let manager = SpiffsManager::new(temp_dir.path().to_path_buf(), false);
        assert!(!manager.validate_spiffs_setup().unwrap());
    }
}
