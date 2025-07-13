//! Arduino Deployer Library
//! 
//! Simple library to deploy Arduino sketches via USB
//! 
//! # Features
//! - List available serial ports
//! - Compile and upload Arduino sketches
//! - Monitor serial output
//! - Support for common Arduino boards (Uno, Nano, Mega, Leonardo)

pub mod utils;

pub use utils::{ArduinoSketch, BoardInfo, check_arduino_cli, create_example_sketch};

use anyhow::Result;
use log::{info, error};
use std::path::PathBuf;
use std::time::Duration;
use std::io::Write;

/// Arduino deployer main structure
pub struct ArduinoDeployer {
    pub verbose: bool,
    pub arduino_cli_path: Option<String>,
}

impl ArduinoDeployer {
    /// Create a new Arduino deployer instance
    pub fn new(verbose: bool) -> Self {
        ArduinoDeployer { 
            verbose,
            arduino_cli_path: None,
        }
    }
    
    /// Set custom Arduino CLI path
    pub fn set_arduino_cli_path(&mut self, path: String) {
        self.arduino_cli_path = Some(path);
    }
    
    /// Get Arduino CLI command
    fn get_arduino_cli_cmd(&self) -> String {
        self.arduino_cli_path.clone().unwrap_or_else(|| "arduino-cli".to_string())
    }
    
    /// List all available serial ports
    pub fn list_ports(&self) -> Result<Vec<String>> {
        info!("🔍 Scanning for available serial ports...");
        
        let ports = serialport::available_ports()?;
        let port_names: Vec<String> = ports.iter()
            .map(|p| p.port_name.clone())
            .collect();
        
        if self.verbose {
            for port in &ports {
                println!("  • {} - {:?}", port.port_name, port.port_type);
            }
        }
        
        Ok(port_names)
    }
    
    /// Auto-detect Arduino port
    pub fn auto_detect_arduino_port(&self) -> Result<Option<String>> {
        info!("🔍 Auto-detecting Arduino port...");
        
        let ports = serialport::available_ports()?;
        
        // Look for Arduino-specific ports
        for port in &ports {
            let port_name = &port.port_name;
            
            // Check port type for Arduino indicators
            match &port.port_type {
                serialport::SerialPortType::UsbPort(usb_info) => {
                    // Common Arduino vendor IDs
                    if usb_info.vid == 0x2341 || // Arduino official
                       usb_info.vid == 0x1A86 || // CH340 (common on Arduino clones)
                       usb_info.vid == 0x0403 || // FTDI
                       usb_info.vid == 0x10C4 {   // Silicon Labs
                        if self.verbose {
                            println!("✅ Arduino detected: {} (VID: {:04X})", port_name, usb_info.vid);
                        }
                        return Ok(Some(port_name.clone()));
                    }
                }
                _ => {}
            }
            
            // Fallback: check port name patterns
            if port_name.contains("Arduino") || 
               port_name.contains("USB") ||
               (cfg!(windows) && port_name.starts_with("COM")) {
                if self.verbose {
                    println!("🔍 Potential Arduino port: {}", port_name);
                }
                return Ok(Some(port_name.clone()));
            }
        }
        
        // If no Arduino-specific port found, return the first available port
        if let Some(first_port) = ports.first() {
            if self.verbose {
                println!("⚠️  No Arduino-specific port found, using first available: {}", first_port.port_name);
            }
            return Ok(Some(first_port.port_name.clone()));
        }
        
        Ok(None)
    }
    
    /// Deploy a sketch to Arduino
    pub fn deploy_sketch(&self, sketch_path: PathBuf, port: &str, board: &str) -> Result<()> {
        info!("🚀 Deploying sketch to Arduino...");
        
        // Load and validate sketch
        let sketch = ArduinoSketch::load_from_file(sketch_path)?;
        sketch.validate()?;
        
        // Get board info
        let board_info = BoardInfo::get_board_info(board);
        
        // Auto-detect port if not specified or if "auto" is passed
        let final_port = if port.is_empty() || port == "auto" {
            match self.auto_detect_arduino_port()? {
                Some(detected_port) => {
                    if self.verbose {
                        println!("� Auto-detected port: {}", detected_port);
                    }
                    detected_port
                }
                None => {
                    return Err(anyhow::anyhow!("No Arduino port could be auto-detected"));
                }
            }
        } else {
            port.to_string()
        };
        
        if self.verbose {
            println!("�📄 Sketch: {}", sketch.name);
            println!("📡 Port: {}", final_port);
            println!("🔧 Board: {} ({})", board_info.name, board_info.fqbn);
        }
        
        // Create temporary Arduino CLI compatible structure
        let temp_sketch_dir = self.create_temp_sketch_structure(&sketch)?;
        
        // Compile and upload using the temp structure
        let result = self.compile_and_upload_temp_sketch(&temp_sketch_dir, &sketch.name, &final_port, &board_info);
        
        // Clean up temporary structure
        if let Err(e) = std::fs::remove_dir_all(&temp_sketch_dir) {
            if self.verbose {
                println!("⚠️  Warning: Could not clean up temp directory: {}", e);
            }
        }
        
        result?;
        
        println!("✅ Deployment completed successfully!");
        Ok(())
    }
    
    /// Create temporary Arduino CLI compatible structure
    fn create_temp_sketch_structure(&self, sketch: &ArduinoSketch) -> Result<PathBuf> {
        use std::fs;
        
        // Create temp directory in the same location as the original sketch
        let temp_dir = std::env::temp_dir().join("arduino_deployer_temp");
        let sketch_temp_dir = temp_dir.join(&sketch.name);
        
        // Remove existing temp directory if it exists
        if sketch_temp_dir.exists() {
            fs::remove_dir_all(&sketch_temp_dir)?;
        }
        
        // Create the directory structure
        fs::create_dir_all(&sketch_temp_dir)?;
        
        // Copy the sketch file with the correct name
        let temp_sketch_file = sketch_temp_dir.join(format!("{}.ino", sketch.name));
        fs::copy(&sketch.path, &temp_sketch_file)?;
        
        if self.verbose {
            println!("📁 Created temp sketch structure: {}", sketch_temp_dir.display());
        }
        
        Ok(sketch_temp_dir)
    }
    
    /// Compile and upload using temporary sketch structure
    fn compile_and_upload_temp_sketch(&self, temp_dir: &PathBuf, sketch_name: &str, port: &str, board_info: &BoardInfo) -> Result<()> {
        // Compile
        info!("🔨 Compiling sketch...");
        let arduino_cli = self.get_arduino_cli_cmd();
        let compile_output = std::process::Command::new(&arduino_cli)
            .args(&["compile", "--fqbn", &board_info.fqbn])
            .arg(temp_dir)
            .output()?;
        
        if !compile_output.status.success() {
            let stderr = String::from_utf8_lossy(&compile_output.stderr);
            error!("❌ Compilation failed:\n{}", stderr);
            return Err(anyhow::anyhow!("Compilation failed"));
        }
        
        if self.verbose {
            println!("✅ Compilation successful");
        }
        
        // Upload
        info!("📤 Uploading sketch...");
        let upload_output = std::process::Command::new(&arduino_cli)
            .args(&["upload", "--fqbn", &board_info.fqbn, "--port", port])
            .arg(temp_dir)
            .output()?;
        
        if !upload_output.status.success() {
            let stderr = String::from_utf8_lossy(&upload_output.stderr);
            error!("❌ Upload failed:\n{}", stderr);
            return Err(anyhow::anyhow!("Upload failed"));
        }
        
        if self.verbose {
            println!("✅ Upload successful");
        }
        
        Ok(())
    }
    
    /// Compile the sketch
    fn compile_sketch(&self, sketch: &ArduinoSketch, board_info: &BoardInfo) -> Result<()> {
        info!("🔨 Compiling sketch...");
        
        // Arduino CLI expects the sketch directory, not the .ino file
        let sketch_dir = sketch.path.parent()
            .ok_or_else(|| anyhow::anyhow!("Could not determine sketch directory"))?;
        
        let arduino_cli = self.get_arduino_cli_cmd();
        let output = std::process::Command::new(&arduino_cli)
            .args(&["compile", "--fqbn", &board_info.fqbn])
            .arg(sketch_dir)
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("❌ Compilation failed:\n{}", stderr);
            return Err(anyhow::anyhow!("Compilation failed"));
        }
        
        if self.verbose {
            println!("✅ Compilation successful");
        }
        
        Ok(())
    }
    
    /// Upload the sketch
    fn upload_sketch(&self, sketch: &ArduinoSketch, port: &str, board_info: &BoardInfo) -> Result<()> {
        info!("📤 Uploading sketch...");
        
        // Arduino CLI expects the sketch directory, not the .ino file
        let sketch_dir = sketch.path.parent()
            .ok_or_else(|| anyhow::anyhow!("Could not determine sketch directory"))?;
        
        let arduino_cli = self.get_arduino_cli_cmd();
        let output = std::process::Command::new(&arduino_cli)
            .args(&["upload", "--fqbn", &board_info.fqbn, "--port", port])
            .arg(sketch_dir)
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("❌ Upload failed:\n{}", stderr);
            return Err(anyhow::anyhow!("Upload failed"));
        }
        
        if self.verbose {
            println!("✅ Upload successful");
        }
        
        Ok(())
    }
    
    /// Monitor serial output
    pub fn monitor_serial(&self, port: &str, baud: u32) -> Result<()> {
        info!("📡 Monitoring serial port {} at {} baud...", port, baud);
        
        let mut serial = serialport::new(port, baud)
            .timeout(Duration::from_millis(1000))
            .open()?;
        
        println!("📡 Serial monitor started (Ctrl+C to exit)");
        println!("Port: {} | Baud: {}", port, baud);
        println!("{}", "─".repeat(50));
        
        let mut buffer = [0; 1024];
        
        loop {
            match serial.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        let data = String::from_utf8_lossy(&buffer[..bytes_read]);
                        print!("{}", data);
                        std::io::stdout().flush()?;
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    continue;
                }
                Err(e) => {
                    error!("❌ Serial read error: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }
}
