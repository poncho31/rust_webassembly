use anyhow::Result;
use std::net::Ipv4Addr;
use std::process::Command;
use std::time::Duration;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// ESP8266 Network Detector and Web Server Inspector
pub struct ESP8266Detector {
    pub verbose: bool,
}

impl ESP8266Detector {
    pub fn new(verbose: bool) -> Self {
        ESP8266Detector { verbose }
    }

    /// Read access token from configuration file
    fn read_access_token(&self) -> Option<String> {
        let config_paths = vec![
            "static/data/wifi_config.json",
            "data/wifi_config.json", 
            "wifi_config.json",
        ];
        
        for path in config_paths {
            if Path::new(path).exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(config) = serde_json::from_str::<Value>(&content) {
                        if let Some(security) = config.get("security") {
                            if let Some(token) = security.get("access_token") {
                                if let Some(token_str) = token.as_str() {
                                    if !token_str.is_empty() && token_str != "YOUR_SECRET_TOKEN_HERE" {
                                        if self.verbose {
                                            println!("🔑 Using access token from configuration");
                                        }
                                        return Some(token_str.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if self.verbose {
            println!("⚠️  No valid access token found in configuration");
        }
        None
    }
    
    /// Scan the local network for ESP8266 devices
    pub fn scan_network(&self) -> Result<Vec<String>> {
        if self.verbose {
            println!("🔍 Scanning local network for ESP8266 devices...");
        }
        
        let mut devices = Vec::new();
        
        // Get local IP range
        let local_ip = self.get_local_ip()?;
        let network_base = format!("{}.{}.{}", 
            local_ip.octets()[0], 
            local_ip.octets()[1], 
            local_ip.octets()[2]
        );
        
        if self.verbose {
            println!("📡 Scanning network range: {}.1-254", network_base);
        }
        
        // Scan common ESP8266 ports (80, 8080)
        for i in 1..=254 {
            let ip = format!("{}.{}", network_base, i);
            
            // Try to connect to port 80 (HTTP)
            if self.check_http_port(&ip, 80).is_ok() {
                if self.verbose {
                    println!("✅ Found HTTP server at: {}", ip);
                }
                devices.push(ip);
            }
        }
        
        Ok(devices)
    }
    
    /// Get local IP address
    fn get_local_ip(&self) -> Result<Ipv4Addr> {
        // Try to get local IP using ipconfig on Windows
        let output = Command::new("ipconfig")
            .output()?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Look for IPv4 address
        for line in output_str.lines() {
            if line.contains("IPv4") && line.contains("192.168.") {
                if let Some(ip_part) = line.split(':').nth(1) {
                    let ip_str = ip_part.trim();
                    if let Ok(ip) = ip_str.parse::<Ipv4Addr>() {
                        return Ok(ip);
                    }
                }
            }
        }
        
        // Fallback to common network
        Ok(Ipv4Addr::new(192, 168, 1, 1))
    }
    
    /// Check if HTTP port is open
    fn check_http_port(&self, ip: &str, port: u16) -> Result<()> {
        use std::net::TcpStream;
        
        let address = format!("{}:{}", ip, port);
        let _stream = TcpStream::connect_timeout(
            &address.parse()?,
            Duration::from_millis(500)
        )?;
        
        Ok(())
    }
    
    /// Test ESP8266 web server and get information
    pub fn inspect_esp8266(&self, ip: &str) -> Result<ESP8266Info> {
        if self.verbose {
            println!("🔍 Inspecting ESP8266 at: {}", ip);
        }
        
        let mut info = ESP8266Info {
            ip: ip.to_string(),
            status: "Unknown".to_string(),
            device_name: "Unknown".to_string(),
            version: "Unknown".to_string(),
            uptime: 0,
            free_heap: 0,
            wifi_rssi: 0,
            endpoints: Vec::new(),
            led_state: false,
            relay_state: false,
            analog_value: 0,
        };
        
        // Try to get status from API
        let token = self.read_access_token();
        let api_url = if let Some(access_token) = token {
            let encoded_token = urlencoding::encode(&access_token);
            format!("http://{}/api/status?token={}", ip, encoded_token)
        } else {
            format!("http://{}/api/status", ip)
        };
        
        match self.get_json_from_url(&api_url) {
            Ok(status_json) => {
                if let Ok(status) = serde_json::from_str::<Value>(&status_json) {
                    info.status = "Connected".to_string();
                    
                    if let Some(device_name) = status.get("device_name").and_then(|v| v.as_str()) {
                        info.device_name = device_name.to_string();
                    }
                    
                    if let Some(version) = status.get("version").and_then(|v| v.as_str()) {
                        info.version = version.to_string();
                    }
                    
                    if let Some(uptime) = status.get("uptime").and_then(|v| v.as_u64()) {
                        info.uptime = uptime;
                    }
                    
                    if let Some(free_heap) = status.get("free_heap").and_then(|v| v.as_u64()) {
                        info.free_heap = free_heap;
                    }
                    
                    if let Some(wifi_rssi) = status.get("wifi_rssi").and_then(|v| v.as_i64()) {
                        info.wifi_rssi = wifi_rssi;
                    }
                    
                    if let Some(led_state) = status.get("led_state").and_then(|v| v.as_bool()) {
                        info.led_state = led_state;
                    }
                    
                    if let Some(relay_state) = status.get("relay_state").and_then(|v| v.as_bool()) {
                        info.relay_state = relay_state;
                    }
                    
                    if let Some(analog_value) = status.get("analog_value").and_then(|v| v.as_u64()) {
                        info.analog_value = analog_value;
                    }
                }
            }
            Err(_) => {
                // Try to get basic HTML
                if let Ok(_) = self.get_html_from_url(&format!("http://{}", ip)) {
                    info.status = "Web Server Active".to_string();
                }
            }
        }
        
        // Check available endpoints
        info.endpoints = self.probe_endpoints(ip);
        
        Ok(info)
    }
    
    /// Get JSON from URL
    fn get_json_from_url(&self, url: &str) -> Result<String> {
        let response = ureq::get(url)
            .timeout(Duration::from_secs(5))
            .call()?;
        
        Ok(response.into_string()?)
    }
    
    /// Get HTML from URL
    fn get_html_from_url(&self, url: &str) -> Result<String> {
        let response = ureq::get(url)
            .timeout(Duration::from_secs(5))
            .call()?;
        
        Ok(response.into_string()?)
    }
    
    /// Probe available endpoints
    fn probe_endpoints(&self, ip: &str) -> Vec<String> {
        let endpoints = vec![
            "/",
            "/api/status",
            "/api/control",
            "/api/system",
            "/api/wifi",
            "/led/toggle",
            "/relay/toggle",
        ];
        
        let mut available = Vec::new();
        
        for endpoint in endpoints {
            let url = format!("http://{}{}", ip, endpoint);
            if let Ok(_) = self.get_html_from_url(&url) {
                available.push(endpoint.to_string());
            }
        }
        
        available
    }
    
    /// Control ESP8266 LED
    pub fn control_led(&self, ip: &str, action: &str) -> Result<String> {
        let url = format!("http://{}/led/{}", ip, action);
        let response = ureq::get(&url)
            .timeout(Duration::from_secs(5))
            .call()?;
        
        Ok(response.into_string()?)
    }
    
    /// Control ESP8266 Relay
    pub fn control_relay(&self, ip: &str, action: &str) -> Result<String> {
        let url = format!("http://{}/relay/{}", ip, action);
        let response = ureq::get(&url)
            .timeout(Duration::from_secs(5))
            .call()?;
        
        Ok(response.into_string()?)
    }
    
    /// Get real-time system information
    pub fn get_system_info(&self, ip: &str) -> Result<Value> {
        let url = format!("http://{}/api/system", ip);
        let response = self.get_json_from_url(&url)?;
        
        Ok(serde_json::from_str(&response)?)
    }
    
    /// Get WiFi information
    pub fn get_wifi_info(&self, ip: &str) -> Result<Value> {
        let url = format!("http://{}/api/wifi", ip);
        let response = self.get_json_from_url(&url)?;
        
        Ok(serde_json::from_str(&response)?)
    }
      /// Open web interface in browser
    pub fn open_web_interface(&self, ip: &str) -> Result<()> {
        // Read access token from configuration
        let token = self.read_access_token();
        
        let url = if let Some(access_token) = token {
            // URL encode the token for safe transmission
            let encoded_token = urlencoding::encode(&access_token);
            format!("http://{}/?token={}", ip, encoded_token)
        } else {
            format!("http://{}", ip)
        };
        
        if self.verbose {
            println!("🌐 Opening web interface: {}", url);
        }
        
        // Try to open in default browser
        #[cfg(windows)]
        {
            Command::new("cmd")
                .args(&["/C", "start", &url])
                .spawn()?;
        }
        
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(&url)
                .spawn()?;
        }
        
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(&url)
                .spawn()?;
        }

        if self.verbose {
            println!("🌐 Opening web interface at {}", url);
        }
        
        Ok(())
    }
}

/// ESP8266 device information
#[derive(Debug)]
pub struct ESP8266Info {
    pub ip: String,
    pub status: String,
    pub device_name: String,
    pub version: String,
    pub uptime: u64,
    pub free_heap: u64,
    pub wifi_rssi: i64,
    pub endpoints: Vec<String>,
    pub led_state: bool,
    pub relay_state: bool,
    pub analog_value: u64,
}

impl ESP8266Info {
    /// Display formatted information
    pub fn display(&self) {
        println!("🌐 ESP8266 Device Information");
        println!("══════════════════════════════");
        println!("📡 IP Address: {}", self.ip);
        println!("🔌 Status: {}", self.status);
        println!("📱 Device Name: {}", self.device_name);
        println!("📋 Version: {}", self.version);
        println!("⏱️  Uptime: {} seconds", self.uptime);
        println!("💾 Free Heap: {} bytes", self.free_heap);
        println!("📶 WiFi RSSI: {} dBm", self.wifi_rssi);
        println!("💡 LED State: {}", if self.led_state { "ON" } else { "OFF" });
        println!("🔌 Relay State: {}", if self.relay_state { "ON" } else { "OFF" });
        println!("📊 Analog Value: {}", self.analog_value);
        println!("🛠️  Available Endpoints:");
        for endpoint in &self.endpoints {
            println!("   • {}", endpoint);
        }
        println!("🌐 Web Interface: http://{}", self.ip);
        println!("══════════════════════════════");
    }
}
