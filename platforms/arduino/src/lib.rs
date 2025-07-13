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
pub mod esp8266_detector;

pub use utils::{ArduinoSketch, BoardInfo, check_arduino_cli, create_example_sketch};
pub use esp8266_detector::{ESP8266Detector, ESP8266Info};

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
    
    /// Auto-install ESP8266 support if needed
    pub fn ensure_esp8266_support(&self) -> Result<()> {
        let arduino_cli = self.get_arduino_cli_cmd();
        
        // Check if ESP8266 is already installed
        let list_output = std::process::Command::new(&arduino_cli)
            .args(&["core", "list"])
            .output()?;
        
        let output_str = String::from_utf8_lossy(&list_output.stdout);
        
        if output_str.contains("esp8266:esp8266") {
            if self.verbose {
                println!("✅ ESP8266 support already installed");
            }
            return Ok(());
        }
        
        if self.verbose {
            println!("🔄 Installing ESP8266 support...");
        }
        
        // Update index
        let _update_output = std::process::Command::new(&arduino_cli)
            .args(&["core", "update-index"])
            .output()?;
        
        // Add ESP8266 board manager URL
        let _config_output = std::process::Command::new(&arduino_cli)
            .args(&["config", "add", "board_manager.additional_urls", "http://arduino.esp8266.com/stable/package_esp8266com_index.json"])
            .output()?;
        
        // Update index again
        let _update_output2 = std::process::Command::new(&arduino_cli)
            .args(&["core", "update-index"])
            .output()?;
        
        // Install ESP8266 core
        let install_output = std::process::Command::new(&arduino_cli)
            .args(&["core", "install", "esp8266:esp8266"])
            .output()?;
        
        if !install_output.status.success() {
            let stderr = String::from_utf8_lossy(&install_output.stderr);
            error!("❌ ESP8266 installation failed:\n{}", stderr);
            return Err(anyhow::anyhow!("ESP8266 installation failed"));
        }
        
        if self.verbose {
            println!("✅ ESP8266 support installed successfully");
        }
        
        Ok(())
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
        
        // Auto-install ESP8266 core if deploying ESP8266 sketch
        if board.contains("esp8266") {
            self.ensure_esp8266_core_installed()?;
        }
        
        // Load and validate sketch
        let sketch = ArduinoSketch::load_from_file(sketch_path)?;
        sketch.validate()?;
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
    
    /// Ensure ESP8266 core is installed
    fn ensure_esp8266_core_installed(&self) -> Result<()> {
        let arduino_cli = self.get_arduino_cli_cmd();
        
        // Check if ESP8266 core is already installed
        let list_output = std::process::Command::new(&arduino_cli)
            .args(&["core", "list"])
            .output()?;
        
        let list_output_str = String::from_utf8_lossy(&list_output.stdout);
        
        if list_output_str.contains("esp8266:esp8266") {
            if self.verbose {
                println!("✅ ESP8266 core already installed");
            }
            return Ok(());
        }
        
        println!("🔄 ESP8266 core not found, installing automatically...");
        
        // Add ESP8266 package URL
        println!("📦 Adding ESP8266 package URL...");
        let config_output = std::process::Command::new(&arduino_cli)
            .args(&["config", "add", "board_manager.additional_urls", "http://arduino.esp8266.com/stable/package_esp8266com_index.json"])
            .output()?;
        
        if !config_output.status.success() {
            // Config might already exist, continue
            if self.verbose {
                println!("⚠️  Config might already exist, continuing...");
            }
        }
        
        // Update package index
        println!("🔄 Updating package index...");
        let update_output = std::process::Command::new(&arduino_cli)
            .args(&["core", "update-index"])
            .output()?;
        
        if !update_output.status.success() {
            let stderr = String::from_utf8_lossy(&update_output.stderr);
            error!("❌ Failed to update package index: {}", stderr);
            return Err(anyhow::anyhow!("Failed to update package index"));
        }
        
        // Install ESP8266 core
        println!("📦 Installing ESP8266 core (this may take a few minutes)...");
        let install_output = std::process::Command::new(&arduino_cli)
            .args(&["core", "install", "esp8266:esp8266"])
            .output()?;
        
        if !install_output.status.success() {
            let stderr = String::from_utf8_lossy(&install_output.stderr);
            error!("❌ Failed to install ESP8266 core: {}", stderr);
            return Err(anyhow::anyhow!("Failed to install ESP8266 core"));
        }
        
        println!("✅ ESP8266 core installed successfully!");
        Ok(())
    }
    
    /// Monitor serial and extract IP address
    pub fn monitor_and_get_ip(&self, port: &str, baud: u32, timeout_seconds: u64) -> Result<Option<String>> {
        info!("📡 Monitoring serial for IP address...");
        
        let mut serial = serialport::new(port, baud)
            .timeout(Duration::from_millis(1000))
            .open()?;
        
        let mut buffer = [0; 1024];
        let start_time = std::time::Instant::now();
        let mut full_output = String::new();
        
        println!("📡 Monitoring serial output for IP address...");
        
        loop {
            // Check timeout
            if start_time.elapsed().as_secs() > timeout_seconds {
                println!("⏰ Timeout reached, stopping monitor");
                break;
            }
            
            match serial.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        let data = String::from_utf8_lossy(&buffer[..bytes_read]);
                        print!("{}", data);
                        std::io::stdout().flush()?;
                        
                        full_output.push_str(&data);
                        
                        // Look for IP address pattern
                        if let Some(ip) = self.extract_ip_from_output(&full_output) {
                            println!("\n🌐 Found IP address: {}", ip);
                            return Ok(Some(ip));
                        }
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
        
        Ok(None)
    }
    
    /// Extract IP address from serial output
    fn extract_ip_from_output(&self, output: &str) -> Option<String> {
        // Look for IP address patterns in the output
        let ip_patterns = [
            r"WEB SERVER READY: http://(\d+\.\d+\.\d+\.\d+)",
            r"IP address: (\d+\.\d+\.\d+\.\d+)",
            r"http://(\d+\.\d+\.\d+\.\d+)",
        ];
        
        for pattern in &ip_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(output) {
                    if let Some(ip) = caps.get(1) {
                        return Some(ip.as_str().to_string());
                    }
                }
            }
        }
        
        None
    }
    
    /// Open web browser with the given URL
    pub fn open_browser(&self, url: &str) -> Result<()> {
        if self.verbose {
            println!("🌐 Opening web browser: {}", url);
        }
        
        #[cfg(windows)]
        {
            std::process::Command::new("cmd")
                .args(&["/C", "start", url])
                .spawn()?;
        }
        
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(url)
                .spawn()?;
        }
        
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(url)
                .spawn()?;
        }
        
        Ok(())
    }
    
    /// Deploy sketch with auto web opening
    pub fn deploy_and_open_web(&self, sketch_path: PathBuf, port: &str, board: &str) -> Result<()> {
        // Deploy the sketch
        self.deploy_sketch(sketch_path, port, board)?;
        
        // If it's an ESP8266 sketch, monitor for IP and open browser
        if board.contains("esp8266") {
            println!("🔄 Waiting for ESP8266 to start and connect to WiFi...");
            
            if let Some(ip) = self.monitor_and_get_ip(port, 115200, 30)? {
                let url = format!("http://{}", ip);
                
                println!("🌐 ESP8266 Web Server is ready!");
                println!("🌐 URL: {}", url);
                
                // Wait a moment for the web server to fully start
                std::thread::sleep(Duration::from_secs(2));
                
                // Open browser
                self.open_browser(&url)?;
                
                println!("✅ Web interface opened in browser!");
            } else {
                println!("⚠️  Could not detect IP address from serial output");
                println!("💡 Check serial monitor manually for IP address");
            }
        }
        
        Ok(())
    }
}
