//! Arduino Deployment Automation
//! 
//! This module handles the complete deployment workflow for Arduino projects
//! Previously handled by run_arduino.bat

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use log::error;
use crate::spiffs_manager::SpiffsManager;
use crate::esp8266_web_tester::ESP8266WebTester;
use crate::ArduinoDeployer;

pub struct ArduinoDeploymentManager {
    pub verbose: bool,
    pub work_dir: PathBuf,
    pub arduino_cli_path: Option<String>,
}

impl ArduinoDeploymentManager {
    pub fn new(work_dir: PathBuf, verbose: bool) -> Self {
        ArduinoDeploymentManager {
            verbose,
            work_dir,
            arduino_cli_path: None,
        }
    }

    pub fn set_arduino_cli_path(&mut self, path: String) {
        self.arduino_cli_path = Some(path);
    }

    /// Auto-deploy Arduino sketch with full workflow
    pub fn auto_deploy_sketch(&self, sketch_path: &Path, port: &str, board: Option<&str>) -> Result<()> {
        if self.verbose {
            println!("===============================================");
            println!("     Arduino Auto Deployment Workflow");
            println!("===============================================");
        }

        // Validate sketch file
        if !sketch_path.exists() {
            error!("❌ Sketch file not found: {}", sketch_path.display());
            self.list_available_sketches()?;
            return Err(anyhow::anyhow!("Sketch file not found"));
        }

        // Auto-detect board type from sketch filename
        let board_type = if let Some(board) = board {
            board.to_string()
        } else {
            self.detect_board_type_from_sketch(sketch_path)?
        };

        if self.verbose {
            println!("Configuration:");
            println!("  Sketch: {}", sketch_path.display());
            println!("  Port: {}", port);
            println!("  Board: {}", board_type);
            println!();
        }

        // Check if ESP8266 deployment
        let is_esp8266 = self.is_esp8266_board(&board_type);
        
        if is_esp8266 {
            self.deploy_esp8266_with_web_interface(sketch_path, port, &board_type)?;
        } else {
            self.deploy_standard_arduino(sketch_path, port, &board_type)?;
        }

        Ok(())
    }

    /// Deploy ESP8266 with web interface setup
    fn deploy_esp8266_with_web_interface(&self, sketch_path: &Path, port: &str, board: &str) -> Result<()> {
        if self.verbose {
            println!("🔄 ESP8266 sketch detected - Using auto-deployment with web interface");
        }

        // Setup SPIFFS automatically if needed
        let spiffs_manager = SpiffsManager::new(self.work_dir.clone(), self.verbose);
        spiffs_manager.auto_setup_if_needed()
            .context("Failed to setup SPIFFS data")?;

        // Deploy with web interface support
        self.deploy_with_web_support(sketch_path, port, board)?;

        // Wait for ESP8266 to boot
        if self.verbose {
            println!("⏳ Waiting for ESP8266 to boot...");
        }
        std::thread::sleep(Duration::from_secs(5));

        // Test and open web interface
        self.test_and_open_web_interface()?;

        // Start serial monitor
        self.start_serial_monitor(port, 115200)?;

        Ok(())
    }

    /// Deploy standard Arduino sketch
    fn deploy_standard_arduino(&self, sketch_path: &Path, port: &str, board: &str) -> Result<()> {
        if self.verbose {
            println!("🔄 Standard Arduino deployment");
        }

        let mut deployer = ArduinoDeployer::new(self.verbose);
        if let Some(ref cli_path) = self.arduino_cli_path {
            deployer.set_arduino_cli_path(cli_path.clone());
        }

        deployer.deploy_sketch(sketch_path.to_path_buf(), port, board)?;

        if self.verbose {
            println!("✅ Deployment completed successfully!");
        }

        // Start serial monitor with standard baud rate
        self.start_serial_monitor(port, 9600)?;

        Ok(())
    }

    /// Deploy with web interface support
    fn deploy_with_web_support(&self, sketch_path: &Path, port: &str, board: &str) -> Result<()> {
        if self.verbose {
            println!("🚀 Starting ESP8266 deployment...");
        }

        let mut deployer = ArduinoDeployer::new(self.verbose);
        if let Some(ref cli_path) = self.arduino_cli_path {
            deployer.set_arduino_cli_path(cli_path.clone());
        }

        // Ensure ESP8266 support is installed
        deployer.ensure_esp8266_support()?;

        // Deploy the sketch
        deployer.deploy_sketch(sketch_path.to_path_buf(), port, board)?;

        if self.verbose {
            println!("✅ Deployment completed successfully!");
        }

        Ok(())
    }

    /// Test ESP8266 and open web interface
    fn test_and_open_web_interface(&self) -> Result<()> {
        let tester = ESP8266WebTester::new(self.verbose);
        
        if self.verbose {
            println!("🔍 Scanning for ESP8266 devices...");
        }

        let devices = tester.auto_detect_esp8266()?;
        
        if devices.is_empty() {
            if self.verbose {
                println!("❌ No ESP8266 found automatically. Please check your device and network connection.");
                println!("🌐 If your ESP8266 is connected, check Serial Monitor for IP address.");
            }
            return Ok(());
        }

        // Test the first found device
        for device in &devices {
            let result = tester.test_esp8266_server(device)?;
            
            if result.is_fully_functional() {
                if self.verbose {
                    println!("✅ ESP8266 fully functional at: {}", device);
                }
                
                // Open web interface
                tester.open_web_interface(device)?;
                return Ok(());
            }
        }

        if self.verbose {
            println!("⚠️  ESP8266 found but not fully functional. Check the device status.");
        }

        Ok(())
    }

    /// Start serial monitor with specified baud rate
    fn start_serial_monitor(&self, port: &str, baud: u32) -> Result<()> {
        if self.verbose {
            println!("📡 Starting serial monitor ({} baud)...", baud);
            println!("   Press Ctrl+C to stop monitoring");
            println!();
        }

        // Small delay before starting monitor
        std::thread::sleep(Duration::from_secs(2));

        let mut deployer = ArduinoDeployer::new(self.verbose);
        if let Some(ref cli_path) = self.arduino_cli_path {
            deployer.set_arduino_cli_path(cli_path.clone());
        }

        deployer.monitor_serial(port, baud)?;

        Ok(())
    }

    /// Detect board type from sketch filename
    fn detect_board_type_from_sketch(&self, sketch_path: &Path) -> Result<String> {
        let filename = sketch_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        if filename.contains("esp8266") {
            Ok("esp8266:esp8266:nodemcuv2".to_string())
        } else if filename.contains("esp32") {
            Ok("esp32:esp32:esp32dev".to_string())
        } else if filename.contains("nano") {
            Ok("arduino:avr:nano".to_string())
        } else if filename.contains("mega") {
            Ok("arduino:avr:mega".to_string())
        } else if filename.contains("leonardo") {
            Ok("arduino:avr:leonardo".to_string())
        } else {
            Ok("arduino:avr:uno".to_string()) // Default to Uno
        }
    }

    /// Check if board is ESP8266
    fn is_esp8266_board(&self, board: &str) -> bool {
        board.contains("esp8266")
    }

    /// List available sketches in static directory
    fn list_available_sketches(&self) -> Result<()> {
        let static_dir = self.work_dir.join("static");
        
        if !static_dir.exists() {
            return Ok(());
        }

        if self.verbose {
            println!("Available sketches:");
        }

        for entry in std::fs::read_dir(&static_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "ino") {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if self.verbose {
                        println!("  • {}", filename);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get deployment configuration for different scenarios
    pub fn get_deployment_config(&self, sketch_path: &Path) -> Result<DeploymentConfig> {
        let board_type = self.detect_board_type_from_sketch(sketch_path)?;
        let is_esp8266 = self.is_esp8266_board(&board_type);

        let config = DeploymentConfig {
            board_type,
            is_esp8266,
            baud_rate: if is_esp8266 { 115200 } else { 9600 },
            requires_spiffs: is_esp8266,
            supports_web_interface: is_esp8266,
            upload_speed: if is_esp8266 { 921600 } else { 115200 },
        };

        Ok(config)
    }

    /// Validate deployment environment
    pub fn validate_deployment_environment(&self) -> Result<()> {
        // Check if arduino-cli is available
        let arduino_cli = self.arduino_cli_path.as_ref()
            .map(|p| p.as_str())
            .unwrap_or("arduino-cli");

        let output = Command::new(arduino_cli)
            .arg("version")
            .output()
            .context("Failed to execute arduino-cli. Make sure it's installed and in PATH")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("arduino-cli is not working properly"));
        }

        if self.verbose {
            println!("✅ Arduino CLI available");
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DeploymentConfig {
    pub board_type: String,
    pub is_esp8266: bool,
    pub baud_rate: u32,
    pub requires_spiffs: bool,
    pub supports_web_interface: bool,
    pub upload_speed: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_deployment_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let manager = ArduinoDeploymentManager::new(temp_dir.path().to_path_buf(), false);
        assert_eq!(manager.work_dir, temp_dir.path().to_path_buf());
    }

    #[test]
    fn test_board_type_detection() {
        let temp_dir = tempdir().unwrap();
        let manager = ArduinoDeploymentManager::new(temp_dir.path().to_path_buf(), false);
        
        let esp8266_path = temp_dir.path().join("esp8266_webserver.ino");
        std::fs::write(&esp8266_path, "// ESP8266 sketch").unwrap();
        
        let board_type = manager.detect_board_type_from_sketch(&esp8266_path).unwrap();
        assert_eq!(board_type, "esp8266:esp8266:nodemcuv2");
    }

    #[test]
    fn test_esp8266_detection() {
        let temp_dir = tempdir().unwrap();
        let manager = ArduinoDeploymentManager::new(temp_dir.path().to_path_buf(), false);
        
        assert!(manager.is_esp8266_board("esp8266:esp8266:nodemcuv2"));
        assert!(!manager.is_esp8266_board("arduino:avr:uno"));
    }
}
