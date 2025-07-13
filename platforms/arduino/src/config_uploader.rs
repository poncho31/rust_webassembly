//! Configuration File Uploader for ESP8266 SPIFFS
//! 
//! This module handles uploading configuration files directly to ESP8266 SPIFFS
//! via serial communication in pure Rust.

use anyhow::{Result, Context};
use std::path::PathBuf;
use std::time::Duration;
use std::io::Write;
use serialport;

pub struct ConfigUploader {
    pub verbose: bool,
    pub timeout: Duration,
}

impl ConfigUploader {
    pub fn new(verbose: bool) -> Self {
        ConfigUploader {
            verbose,
            timeout: Duration::from_secs(10),
        }
    }
    
    /// Upload configuration file to ESP8266 SPIFFS
    pub fn upload_config_file(&self, port: &str, config_file: &PathBuf, baud: u32) -> Result<()> {
        // Read configuration file
        let config_content = std::fs::read_to_string(config_file)
            .with_context(|| format!("Failed to read config file: {}", config_file.display()))?;
        
        if self.verbose {
            println!("📄 Config file content ({} bytes):", config_content.len());
            println!("{}", config_content);
        }
        
        // Upload via serial
        self.upload_file_via_serial(port, "/wifi_config.json", &config_content, baud)
    }
    
    /// Upload HTML file to ESP8266 SPIFFS
    pub fn upload_html_file(&self, port: &str, html_file: &PathBuf, baud: u32) -> Result<()> {
        // Read HTML file
        let html_content = std::fs::read_to_string(html_file)
            .with_context(|| format!("Failed to read HTML file: {}", html_file.display()))?;
        
        if self.verbose {
            println!("🌐 HTML file content ({} bytes)", html_content.len());
        }
        
        // Upload via serial
        self.upload_file_via_serial(port, "/arduino.html", &html_content, baud)
    }
    
    /// Upload file content via serial communication
    fn upload_file_via_serial(&self, port: &str, spiffs_path: &str, content: &str, baud: u32) -> Result<()> {
        if self.verbose {
            println!("📡 Opening serial port: {} at {} baud", port, baud);
        }
        
        let mut serial = serialport::new(port, baud)
            .timeout(self.timeout)
            .open()
            .with_context(|| format!("Failed to open serial port: {}", port))?;
        
        // Wait for ESP8266 to be ready
        std::thread::sleep(Duration::from_millis(1000));
        
        // Send file upload command
        let upload_command = format!("UPLOAD_FILE:{}\n", spiffs_path);
        
        if self.verbose {
            println!("📤 Sending upload command: {}", upload_command.trim());
        }
        
        serial.write_all(upload_command.as_bytes())
            .context("Failed to send upload command")?;
        
        // Wait for confirmation
        std::thread::sleep(Duration::from_millis(500));
        
        // Send file size
        let size_command = format!("SIZE:{}\n", content.len());
        
        if self.verbose {
            println!("📊 Sending file size: {}", content.len());
        }
        
        serial.write_all(size_command.as_bytes())
            .context("Failed to send file size")?;
        
        // Wait for confirmation
        std::thread::sleep(Duration::from_millis(500));
        
        // Send file content in chunks
        let chunk_size = 512;
        let chunks: Vec<&str> = content.as_bytes()
            .chunks(chunk_size)
            .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
            .collect();
        
        if self.verbose {
            println!("📦 Sending {} chunks of max {} bytes", chunks.len(), chunk_size);
        }
        
        for (i, chunk) in chunks.iter().enumerate() {
            if self.verbose {
                println!("📦 Sending chunk {}/{} ({} bytes)", i + 1, chunks.len(), chunk.len());
            }
            
            serial.write_all(chunk.as_bytes())
                .context("Failed to send file chunk")?;
            
            // Small delay between chunks
            std::thread::sleep(Duration::from_millis(100));
        }
        
        // Send end marker
        serial.write_all(b"END_UPLOAD\n")
            .context("Failed to send end marker")?;
        
        if self.verbose {
            println!("✅ File upload completed");
        }
        
        // Wait for ESP8266 to process
        std::thread::sleep(Duration::from_millis(1000));
        
        Ok(())
    }
    
    /// Generate ESP8266 code to handle serial file uploads
    pub fn generate_upload_handler_code(&self) -> String {
        r#"
// Serial File Upload Handler (add this to your ESP8266 sketch)
void handleSerialUpload() {
  if (Serial.available()) {
    String command = Serial.readStringUntil('\n');
    command.trim();
    
    if (command.startsWith("UPLOAD_FILE:")) {
      String filePath = command.substring(12);
      Serial.print("READY_FOR_FILE:");
      Serial.println(filePath);
      
      // Wait for file size
      while (!Serial.available()) {
        delay(10);
      }
      
      String sizeCommand = Serial.readStringUntil('\n');
      sizeCommand.trim();
      
      if (sizeCommand.startsWith("SIZE:")) {
        int fileSize = sizeCommand.substring(5).toInt();
        Serial.print("READY_FOR_DATA:");
        Serial.println(fileSize);
        
        // Prepare to receive file data
        String fileData = "";
        fileData.reserve(fileSize + 100);
        
        unsigned long startTime = millis();
        while (fileData.length() < fileSize && (millis() - startTime) < 30000) {
          if (Serial.available()) {
            char c = Serial.read();
            if (c != '\n' && c != '\r') {
              fileData += c;
            }
          }
        }
        
        // Check for end marker
        String endMarker = "";
        while (Serial.available()) {
          endMarker += (char)Serial.read();
        }
        
        if (endMarker.indexOf("END_UPLOAD") >= 0) {
          // Save file to SPIFFS
          File file = SPIFFS.open(filePath, "w");
          if (file) {
            file.print(fileData);
            file.close();
            Serial.print("UPLOAD_SUCCESS:");
            Serial.println(filePath);
          } else {
            Serial.print("UPLOAD_ERROR:");
            Serial.println(filePath);
          }
        }
      }
    }
  }
}

// Add this to your main loop()
void loop() {
  // Your existing code...
  
  // Handle serial uploads
  handleSerialUpload();
  
  // Your existing code...
}
"#.to_string()
    }
}
