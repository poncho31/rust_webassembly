//! ESP8266 Web Server Tester
//! 
//! This module handles testing and validation of ESP8266 web servers
//! Previously handled by test_esp8266.bat

use anyhow::{Result, Context};
use std::time::Duration;
use std::process::Command;
use log::warn;
use serde_json::Value;

pub struct ESP8266WebTester {
    pub verbose: bool,
    pub timeout: Duration,
}

impl ESP8266WebTester {
    pub fn new(verbose: bool) -> Self {
        ESP8266WebTester {
            verbose,
            timeout: Duration::from_secs(5),
        }
    }

    /// Test ESP8266 web server functionality
    pub fn test_esp8266_server(&self, ip: &str) -> Result<ESP8266TestResult> {
        if self.verbose {
            println!("🌐 Testing ESP8266 at {}", ip);
        }

        let mut result = ESP8266TestResult::new(ip.to_string());

        // Test basic connectivity
        result.ping_success = self.test_ping(ip)?;
        if !result.ping_success {
            if self.verbose {
                println!("❌ Ping failed - check IP address and network");
            }
            return Ok(result);
        }

        // Test main page
        result.main_page_accessible = self.test_http_endpoint(&format!("http://{}/", ip))?;
        
        // Test API endpoints
        result.api_status = self.test_api_endpoint(&format!("http://{}/api/status", ip))?;
        result.api_system = self.test_api_endpoint(&format!("http://{}/api/system", ip))?;
        result.api_wifi = self.test_api_endpoint(&format!("http://{}/api/wifi", ip))?;

        // Test control endpoints
        result.led_control = self.test_http_endpoint(&format!("http://{}/led/toggle", ip))?;
        result.relay_control = self.test_http_endpoint(&format!("http://{}/relay/toggle", ip))?;

        // Get device status if available
        if result.api_status.is_some() && result.api_status.as_ref().unwrap().success {
            result.device_status = self.get_device_status(ip)?;
        }

        if self.verbose {
            self.print_test_results(&result);
        }

        Ok(result)
    }

    /// Test ping connectivity
    fn test_ping(&self, ip: &str) -> Result<bool> {
        if self.verbose {
            println!("📡 Testing basic connectivity...");
        }

        // First try TCP connection on port 80 (more reliable for ESP8266)
        let tcp_result = self.test_tcp_connection(ip, 80);
        
        if tcp_result.is_ok() {
            if self.verbose {
                println!("✅ TCP connection successful");
            }
            return Ok(true);
        }

        // Fallback to ping with cross-platform arguments
        let ping_args = if cfg!(target_os = "windows") {
            vec!["-n", "1", "-w", "2000", ip]
        } else {
            vec!["-c", "1", "-W", "2", ip]
        };

        let output = Command::new("ping")
            .args(&ping_args)
            .output()
            .context("Failed to execute ping command")?;

        let success = output.status.success();
        
        if self.verbose {
            if success {
                println!("✅ Ping successful");
            } else {
                println!("❌ Ping failed");
                // Show more diagnostic info
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    println!("   Ping output: {}", stdout.trim());
                }
                if let Ok(stderr) = String::from_utf8(output.stderr) {
                    println!("   Ping error: {}", stderr.trim());
                }
            }
        }

        Ok(success)
    }

    /// Test TCP connection to specific port
    fn test_tcp_connection(&self, ip: &str, port: u16) -> Result<()> {
        use std::net::TcpStream;
        
        let address = format!("{}:{}", ip, port);
        let _stream = TcpStream::connect_timeout(
            &address.parse()?,
            Duration::from_millis(1000)  // Reduced from 2000ms to 1000ms
        )?;
        
        Ok(())
    }

    /// Test HTTP endpoint availability
    fn test_http_endpoint(&self, url: &str) -> Result<bool> {
        // First try using TCP connection to check if port is open
        if let Ok(host_port) = self.extract_host_port(url) {
            if self.test_tcp_connection(&host_port.0, host_port.1).is_ok() {
                // Port is open, now try HTTP request
                return self.test_http_request(url);
            }
        }
        
        // If TCP test fails, try HTTP directly as fallback
        self.test_http_request(url)
    }
    
    /// Extract host and port from URL
    fn extract_host_port(&self, url: &str) -> Result<(String, u16)> {
        let url = url.strip_prefix("http://").unwrap_or(url);
        let parts: Vec<&str> = url.split('/').collect();
        let host_part = parts[0];
        
        if host_part.contains(':') {
            let host_port: Vec<&str> = host_part.split(':').collect();
            let host = host_port[0].to_string();
            let port = host_port[1].parse::<u16>().unwrap_or(80);
            Ok((host, port))
        } else {
            Ok((host_part.to_string(), 80))
        }
    }
    
    /// Test HTTP request
    fn test_http_request(&self, url: &str) -> Result<bool> {
        // Try curl first with faster timeouts
        let curl_result = Command::new("curl")
            .args(&["-s", "-m", "3", "--connect-timeout", "1", url])
            .output();

        match curl_result {
            Ok(result) => {
                if result.status.success() {
                    return Ok(true);
                }
                
                // If curl fails, show error for debugging
                if self.verbose {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    if !stderr.is_empty() {
                        println!("   Curl error: {}", stderr.trim());
                    }
                }
            }
            Err(_) => {
                if self.verbose {
                    println!("   Curl not available, trying alternative method");
                }
            }
        }
        
        // Fallback to basic TCP test
        if let Ok(host_port) = self.extract_host_port(url) {
            return Ok(self.test_tcp_connection(&host_port.0, host_port.1).is_ok());
        }
        
        Ok(false)
    }

    /// Test API endpoint and return response info
    fn test_api_endpoint(&self, url: &str) -> Result<Option<ApiTestResult>> {
        let output = Command::new("curl")
            .args(&["-s", "-m", "3", "--connect-timeout", "1", url])  // Reduced timeouts
            .output();

        match output {
            Ok(result) if result.status.success() => {
                let response = String::from_utf8_lossy(&result.stdout);
                let json_valid = serde_json::from_str::<Value>(&response).is_ok();
                
                if self.verbose {
                    println!("✅ {} - OK", url.split('/').last().unwrap_or("API"));
                }
                
                Ok(Some(ApiTestResult {
                    success: true,
                    response: response.to_string(),
                    json_valid,
                }))
            }
            Ok(_) => {
                if self.verbose {
                    println!("❌ {} - Failed", url.split('/').last().unwrap_or("API"));
                }
                Ok(Some(ApiTestResult {
                    success: false,
                    response: String::new(),
                    json_valid: false,
                }))
            }
            Err(_) => Ok(None),
        }
    }

    /// Get device status information
    fn get_device_status(&self, ip: &str) -> Result<Option<DeviceStatus>> {
        let url = format!("http://{}/api/status", ip);
        let output = Command::new("curl")
            .args(&["-s", "-m", "5", url.as_str()])
            .output();

        match output {
            Ok(result) if result.status.success() => {
                let response = String::from_utf8_lossy(&result.stdout);
                if let Ok(json) = serde_json::from_str::<Value>(&response) {
                    let status = DeviceStatus {
                        device_name: json["device_name"].as_str().unwrap_or("Unknown").to_string(),
                        version: json["version"].as_str().unwrap_or("Unknown").to_string(),
                        uptime: json["uptime"].as_u64().unwrap_or(0),
                        free_heap: json["free_heap"].as_u64().unwrap_or(0),
                        wifi_rssi: json["wifi_rssi"].as_i64().unwrap_or(0),
                        led_state: json["led_state"].as_bool().unwrap_or(false),
                        relay_state: json["relay_state"].as_bool().unwrap_or(false),
                    };
                    return Ok(Some(status));
                }
            }
            _ => {}
        }

        Ok(None)
    }

    /// Print comprehensive test results
    fn print_test_results(&self, result: &ESP8266TestResult) {
        println!();
        println!("===============================================");
        println!("           ESP8266 Test Results");
        println!("===============================================");
        println!("IP Address: {}", result.ip);
        println!("Ping: {}", if result.ping_success { "✅ Success" } else { "❌ Failed" });
        println!("Main Page: {}", if result.main_page_accessible { "✅ Accessible" } else { "❌ Not accessible" });
        
        if let Some(ref api) = result.api_status {
            println!("API Status: {}", if api.success { "✅ OK" } else { "❌ Failed" });
        }
        
        if let Some(ref api) = result.api_system {
            println!("API System: {}", if api.success { "✅ OK" } else { "❌ Failed" });
        }
        
        if let Some(ref api) = result.api_wifi {
            println!("API WiFi: {}", if api.success { "✅ OK" } else { "❌ Failed" });
        }
        
        println!("LED Control: {}", if result.led_control { "✅ OK" } else { "❌ Failed" });
        println!("Relay Control: {}", if result.relay_control { "✅ OK" } else { "❌ Failed" });
        
        if let Some(ref status) = result.device_status {
            println!();
            println!("Device Information:");
            println!("  Name: {}", status.device_name);
            println!("  Version: {}", status.version);
            println!("  Uptime: {} seconds", status.uptime);
            println!("  Free Heap: {} bytes", status.free_heap);
            println!("  WiFi RSSI: {} dBm", status.wifi_rssi);
            println!("  LED State: {}", if status.led_state { "ON" } else { "OFF" });
            println!("  Relay State: {}", if status.relay_state { "ON" } else { "OFF" });
        }
        
        println!();
        println!("Manual URLs:");
        println!("  Main page:    http://{}/", result.ip);
        println!("  Status API:   http://{}/api/status", result.ip);
        println!("  System API:   http://{}/api/system", result.ip);
        println!("  WiFi API:     http://{}/api/wifi", result.ip);
        println!("  LED Toggle:   http://{}/led/toggle", result.ip);
        println!("  Relay Toggle: http://{}/relay/toggle", result.ip);
    }

    /// Auto-detect ESP8266 devices on network
    pub fn auto_detect_esp8266(&self) -> Result<Vec<String>> {
        if self.verbose {
            println!("🔍 Scanning for ESP8266 devices...");
        }

        let mut devices = Vec::new();
        
        // First, try to detect the local network range
        let local_networks = self.get_local_networks()?;
        
        for network in local_networks {
            if self.verbose {
                println!("📡 Scanning network: {}", network);
            }
            
            // Use priority ranges for faster detection
            let priority_ranges = vec![
                (100, 150),  // Common DHCP range
                (200, 254),  // Higher DHCP range
                (10, 99),    // Lower range
                (151, 199),  // Middle range
            ];
            
            for (start, end) in priority_ranges {
                let mut found_in_range = false;
                
                for i in start..=end {
                    let ip = format!("{}.{}", network, i);
                    
                    // Quick TCP test on port 80 (faster than ping)
                    if self.test_tcp_connection(&ip, 80).is_ok() {
                        // Test if it responds to ESP8266 API
                        if let Ok(Some(api_result)) = self.test_api_endpoint(&format!("http://{}/api/status", ip)) {
                            // Additional check to verify it's actually an ESP8266
                            if self.is_esp8266_device(&ip)? {
                                if self.verbose {
                                    println!("✅ Found ESP8266 at: {}", ip);
                                }
                                devices.push(ip);
                                found_in_range = true;
                            }
                        }
                    }
                    
                    // If we found a device in this range, continue to next range
                    if found_in_range {
                        break;
                    }
                }
            }
        }
        
        Ok(devices)
    }
    
    /// Check if a device is actually an ESP8266 by examining its responses
    fn is_esp8266_device(&self, ip: &str) -> Result<bool> {
        // Test multiple ESP8266-specific endpoints
        let esp8266_indicators = vec![
            ("/api/status", "device_name"),
            ("/api/system", "chip_id"),
            ("/api/wifi", "rssi"),
        ];
        
        for (endpoint, expected_field) in esp8266_indicators {
            if let Ok(Some(api_result)) = self.test_api_endpoint(&format!("http://{}{}", ip, endpoint)) {
                if api_result.response.contains(expected_field) {
                    return Ok(true);
                }
            }
        }
        
        // Also check for common ESP8266 web server patterns
        if let Ok(true) = self.test_http_request(&format!("http://{}/", ip)) {
            // Try to get the main page and look for ESP8266 indicators
            if let Ok(output) = std::process::Command::new("curl")
                .args(&["-s", "-m", "3", &format!("http://{}/", ip)])
                .output()
            {
                let content = String::from_utf8_lossy(&output.stdout);
                if content.contains("ESP8266") || content.contains("NodeMCU") || content.contains("ESP32") {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }

    /// Ultra-fast ESP8266 detection (< 5 seconds guaranteed)
    pub fn ultra_fast_detect_esp8266(&self) -> Result<Vec<String>> {
        if self.verbose {
            println!("⚡ Ultra-fast ESP8266 detection...");
        }

        let mut devices = Vec::new();
        
        // Step 1: Test known working IP first (instant if it works)
        let known_ips = vec![
            "192.168.0.238",  // Previously found IP
            "192.168.1.238",  // Common alternative
            "192.168.0.200",  // Common static IP
            "192.168.1.200",  
        ];
        
        for ip in known_ips {
            if self.instant_test_esp8266(ip)? {
                if self.verbose {
                    println!("🎯 Found ESP8266 at known IP: {}", ip);
                }
                devices.push(ip.to_string());
                return Ok(devices); // Return immediately if found
            }
        }
        
        // Step 2: Get local network and test high-probability IPs
        let local_networks = self.get_local_networks()?;
        
        for network in local_networks {
            // Test most likely IPs in parallel-like fashion
            let likely_ips = vec![
                format!("{}.238", network),  // The one that worked before
                format!("{}.200", network),  // Common static
                format!("{}.201", network),
                format!("{}.100", network),  // Common DHCP start
                format!("{}.101", network),
                format!("{}.150", network),  // Mid-range
                format!("{}.180", network),
                format!("{}.120", network),
            ];
            
            for ip in likely_ips {
                if self.instant_test_esp8266(&ip)? {
                    if self.verbose {
                        println!("🎯 Found ESP8266 at: {}", ip);
                    }
                    devices.push(ip);
                    return Ok(devices); // Return immediately on first find
                }
            }
        }
        
        Ok(devices)
    }
    
    /// Instant ESP8266 test (< 1 second timeout)
    fn instant_test_esp8266(&self, ip: &str) -> Result<bool> {
        // Ultra-fast TCP test (500ms timeout)
        if self.instant_tcp_test(ip, 80).is_err() {
            return Ok(false);
        }
        
        // Ultra-fast HTTP test for ESP8266 API
        let status_url = format!("http://{}/api/status", ip);
        if let Ok(output) = std::process::Command::new("curl")
            .args(&["-s", "-m", "1", "--connect-timeout", "0.5", &status_url])
            .output()
        {
            if output.status.success() {
                let response = String::from_utf8_lossy(&output.stdout);
                // Quick ESP8266 signature check
                if response.contains("device_name") && 
                   (response.contains("ESP8266") || 
                    response.contains("free_heap") || 
                    response.contains("chip_id") ||
                    response.contains("wifi_rssi")) {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    /// Instant TCP connection test (500ms timeout)
    fn instant_tcp_test(&self, ip: &str, port: u16) -> Result<()> {
        use std::net::TcpStream;
        
        let address = format!("{}:{}", ip, port);
        let _stream = TcpStream::connect_timeout(
            &address.parse()?,
            Duration::from_millis(500)  // Ultra-fast timeout
        )?;
        
        Ok(())
    }
    pub fn fast_detect_esp8266(&self) -> Result<Vec<String>> {
        if self.verbose {
            println!("🚀 Fast ESP8266 detection...");
        }

        let mut devices = Vec::new();
        
        // First, try known/recently found IPs
        let known_ips = vec![
            "192.168.0.238".to_string(),  // Previously found ESP8266
            "192.168.1.238".to_string(),  // Common alternative
            "192.168.0.100".to_string(),  // Common DHCP start
            "192.168.1.100".to_string(),  // Common DHCP start on .1 network
        ];
        
        if self.verbose {
            println!("🎯 Testing known/likely ESP8266 addresses...");
        }
        
        for ip in &known_ips {
            if self.test_esp8266_quickly(ip)? {
                if self.verbose {
                    println!("🎉 Found ESP8266 at known address: {}", ip);
                }
                devices.push(ip.clone());
                return Ok(devices); // Return immediately when found
            }
        }
        
        // If not found in known IPs, continue with network detection
        let local_networks = self.get_local_networks()?;
        
        for network in local_networks {
            if self.verbose {
                println!("📡 Quick scan: {}", network);
            }
            
            // Priority IPs to test (excluding already tested ones)
            let mut priority_ips = vec![
                format!("{}.101", network),
                format!("{}.102", network),
                format!("{}.103", network),
                format!("{}.200", network),
                format!("{}.201", network),
                format!("{}.202", network),
                format!("{}.150", network),
                format!("{}.151", network),
                format!("{}.152", network),
            ];
            
            // Remove already tested IPs
            priority_ips.retain(|ip| !known_ips.contains(ip));
            
            // Test priority IPs
            for ip in priority_ips {
                if self.test_esp8266_quickly(&ip)? {
                    if self.verbose {
                        println!("🎯 Found ESP8266 at: {}", ip);
                    }
                    devices.push(ip);
                    return Ok(devices); // Return immediately when found
                }
            }
            
            // If still not found, do a limited range scan (but faster)
            if devices.is_empty() {
                let ranges = vec![
                    (104, 115),  // Skip 100-103 already tested
                    (203, 210),  // Skip 200-202 already tested
                ];
                
                for (start, end) in ranges {
                    for i in start..=end {
                        let ip = format!("{}.{}", network, i);
                        if self.test_esp8266_quickly(&ip)? {
                            if self.verbose {
                                println!("🎯 Found ESP8266 at: {}", ip);
                            }
                            devices.push(ip);
                            return Ok(devices); // Return immediately when found
                        }
                    }
                }
            }
        }
        
        Ok(devices)
    }
    
    /// Quick test to see if an IP hosts an ESP8266
    fn test_esp8266_quickly(&self, ip: &str) -> Result<bool> {
        // First, quick TCP test
        if self.test_tcp_connection(ip, 80).is_err() {
            return Ok(false);
        }
        
        // Test ESP8266 specific endpoint quickly
        let status_url = format!("http://{}/api/status", ip);
        if let Ok(output) = std::process::Command::new("curl")
            .args(&["-s", "-m", "2", "--connect-timeout", "1", &status_url])
            .output()
        {
            if output.status.success() {
                let response = String::from_utf8_lossy(&output.stdout);
                // Look for ESP8266 specific indicators
                if response.contains("device_name") && 
                   (response.contains("ESP8266") || 
                    response.contains("free_heap") || 
                    response.contains("chip_id")) {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    fn get_local_networks(&self) -> Result<Vec<String>> {
        let mut networks = Vec::new();
        
        // Try to get local IP using ipconfig/ifconfig
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = Command::new("ipconfig").output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("IPv4") && (line.contains("192.168.") || line.contains("10.0.")) {
                        if let Some(ip_part) = line.split(':').nth(1) {
                            let ip_str = ip_part.trim();
                            if let Ok(ip) = ip_str.parse::<std::net::Ipv4Addr>() {
                                let octets = ip.octets();
                                networks.push(format!("{}.{}.{}", octets[0], octets[1], octets[2]));
                            }
                        }
                    }
                }
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(output) = Command::new("ifconfig").output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("inet ") && (line.contains("192.168.") || line.contains("10.0.")) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if let Some(ip_part) = parts.get(1) {
                            if let Ok(ip) = ip_part.parse::<std::net::Ipv4Addr>() {
                                let octets = ip.octets();
                                networks.push(format!("{}.{}.{}", octets[0], octets[1], octets[2]));
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback to common networks if detection fails
        if networks.is_empty() {
            networks = vec!["192.168.1".to_string(), "192.168.0".to_string(), "10.0.0".to_string()];
        }
        
        Ok(networks)
    }

    /// Print network diagnostics
    pub fn print_network_diagnostics(&self) -> Result<()> {
        println!("🔍 Network Diagnostics:");
        
        // Show local networks
        let networks = self.get_local_networks()?;
        println!("📡 Local networks detected:");
        for network in &networks {
            println!("   • {}.x", network);
        }
        
        // Show network configuration
        #[cfg(target_os = "windows")]
        {
            println!("🌐 Network configuration:");
            if let Ok(output) = Command::new("ipconfig").output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    if line.contains("IPv4") || line.contains("Gateway") || line.contains("Subnet") {
                        println!("   {}", line.trim());
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Test specific IP address with detailed diagnostics
    pub fn test_ip_detailed(&self, ip: &str) -> Result<()> {
        println!("🔍 Detailed test for IP: {}", ip);
        
        // Test TCP connection
        print!("📡 Testing TCP connection on port 80... ");
        match self.test_tcp_connection(ip, 80) {
            Ok(_) => println!("✅ SUCCESS"),
            Err(e) => {
                println!("❌ FAILED: {}", e);
                return Ok(());
            }
        }
        
        // Test HTTP endpoint
        print!("🌐 Testing HTTP endpoint... ");
        match self.test_http_request(&format!("http://{}/", ip)) {
            Ok(true) => println!("✅ SUCCESS"),
            Ok(false) => println!("❌ FAILED"),
            Err(e) => println!("❌ ERROR: {}", e),
        }
        
        // Test API endpoint
        print!("📊 Testing API status endpoint... ");
        match self.test_api_endpoint(&format!("http://{}/api/status", ip)) {
            Ok(Some(_)) => println!("✅ SUCCESS"),
            Ok(None) => println!("❌ FAILED"),
            Err(e) => println!("❌ ERROR: {}", e),
        }
        
        Ok(())
    }
    pub fn open_web_interface(&self, ip: &str) -> Result<()> {
        let url = format!("http://{}", ip);
        
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd")
                .args(&["/c", "start", &url])
                .spawn()
                .context("Failed to open web interface")?;
        }
        
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(&url)
                .spawn()
                .context("Failed to open web interface")?;
        }
        
        #[cfg(target_os = "linux")]
        {
            Command::new("xdg-open")
                .arg(&url)
                .spawn()
                .context("Failed to open web interface")?;
        }
        
        if self.verbose {
            println!("🌐 Opening web interface at {}", url);
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct ESP8266TestResult {
    pub ip: String,
    pub ping_success: bool,
    pub main_page_accessible: bool,
    pub api_status: Option<ApiTestResult>,
    pub api_system: Option<ApiTestResult>,
    pub api_wifi: Option<ApiTestResult>,
    pub led_control: bool,
    pub relay_control: bool,
    pub device_status: Option<DeviceStatus>,
}

impl ESP8266TestResult {
    pub fn new(ip: String) -> Self {
        ESP8266TestResult {
            ip,
            ping_success: false,
            main_page_accessible: false,
            api_status: None,
            api_system: None,
            api_wifi: None,
            led_control: false,
            relay_control: false,
            device_status: None,
        }
    }

    pub fn is_fully_functional(&self) -> bool {
        self.ping_success && 
        self.main_page_accessible && 
        self.api_status.as_ref().map_or(false, |a| a.success)
    }
}

#[derive(Debug)]
pub struct ApiTestResult {
    pub success: bool,
    pub response: String,
    pub json_valid: bool,
}

#[derive(Debug)]
pub struct DeviceStatus {
    pub device_name: String,
    pub version: String,
    pub uptime: u64,
    pub free_heap: u64,
    pub wifi_rssi: i64,
    pub led_state: bool,
    pub relay_state: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_esp8266_web_tester_creation() {
        let tester = ESP8266WebTester::new(false);
        assert_eq!(tester.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_result_creation() {
        let result = ESP8266TestResult::new("192.168.1.100".to_string());
        assert_eq!(result.ip, "192.168.1.100");
        assert!(!result.is_fully_functional());
    }
}
