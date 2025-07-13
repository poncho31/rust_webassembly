use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::{Result, Context};
use arduino_deployer::{ArduinoDeployer, BoardInfo, check_arduino_cli, create_example_sketch, ESP8266Detector, ArduinoDeploymentManager, SpiffsManager, ESP8266WebTester, ConfigUploader};

#[derive(Parser)]
#[command(name = "arduino-deploy")]
#[command(about = "Simple Arduino code deployer via USB")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available Arduino ports
    List,
    /// Deploy code to Arduino
    Deploy {
        /// Path to the sketch file (.ino)
        #[arg(short, long)]
        sketch: PathBuf,
        /// Serial port (e.g., COM3 on Windows, /dev/ttyUSB0 on Linux)
        #[arg(short, long)]
        port: String,
        /// Arduino board type (default: uno)
        #[arg(short, long, default_value = "uno")]
        board: String,
        /// Path to arduino-cli executable
        #[arg(long)]
        arduino_cli: Option<String>,
    },
    /// Auto-deploy Arduino sketch with full workflow (replaces run_arduino.bat)
    AutoDeploy {
        /// Path to the sketch file (.ino)
        #[arg(short, long, default_value = "static/esp8266_webserver.ino")]
        sketch: PathBuf,
        /// Serial port (e.g., COM5 on Windows)
        #[arg(short, long, default_value = "COM5")]
        port: String,
        /// Arduino board type (auto-detected if not specified)
        #[arg(short, long)]
        board: Option<String>,
        /// Path to arduino-cli executable
        #[arg(long)]
        arduino_cli: Option<String>,
    },
    /// Deploy ESP8266 with auto-setup and web interface opening
    DeployWeb {
        /// Path to the sketch file (.ino)
        #[arg(short, long, default_value = "static/esp8266_webserver.ino")]
        sketch: PathBuf,
        /// Serial port (e.g., COM5 on Windows)
        #[arg(short, long, default_value = "COM5")]
        port: String,
        /// Arduino board type (default: esp8266:esp8266:nodemcuv2)
        #[arg(short, long, default_value = "esp8266:esp8266:nodemcuv2")]
        board: String,
        /// Path to arduino-cli executable
        #[arg(long)]
        arduino_cli: Option<String>,
        /// Skip SPIFFS setup (use if already configured)
        #[arg(long)]
        skip_spiffs: bool,
    },
    /// Monitor serial output
    Monitor {
        /// Serial port to monitor
        #[arg(short, long)]
        port: String,
        /// Baud rate (default: 9600)
        #[arg(short, long, default_value = "9600")]
        baud: u32,
    },
    /// Create example sketch
    Example {
        /// Output path for the example sketch
        #[arg(short, long, default_value = "static/blink.ino")]
        output: PathBuf,
    },
    /// Show supported boards
    Boards,
    /// Check arduino-cli installation
    Check,
    /// Auto-detect Arduino port
    AutoDetect,
    /// Setup SPIFFS for ESP8266 (replaces setup_spiffs.bat)
    SetupSpiffs {
        /// Working directory (default: current directory)
        #[arg(short, long)]
        work_dir: Option<PathBuf>,
    },
    /// Test ESP8266 web server (replaces test_esp8266.bat)
    TestEsp8266 {
        /// IP address of ESP8266 (auto-detect if not specified)
        #[arg(short, long)]
        ip: Option<String>,
        /// Auto-detect and test all ESP8266 devices
        #[arg(short, long)]
        auto_detect: bool,
    },
    /// ESP8266 network scanner and web interface
    Esp8266 {
        #[command(subcommand)]
        esp_command: ESP8266Commands,
    },
    /// Network diagnostics for ESP8266 connectivity
    NetworkDiag {
        /// IP address to test (optional)
        #[arg(short, long)]
        ip: Option<String>,
    },
    /// Upload configuration files to ESP8266 SPIFFS
    UploadConfig {
        /// Serial port (e.g., COM5 on Windows)
        #[arg(short, long, default_value = "COM5")]
        port: String,
        /// Path to configuration file
        #[arg(short, long, default_value = "static/wifi_config.json")]
        config_file: PathBuf,
        /// Path to HTML file (optional)
        #[arg(long)]
        html_file: Option<PathBuf>,
        /// Baud rate for serial communication
        #[arg(short, long, default_value = "115200")]
        baud: u32,
    },
    /// Complete ESP8266 workflow: setup SPIFFS, deploy, upload config, test, and monitor
    Complete {
        /// Path to the sketch file (.ino)
        #[arg(short, long, default_value = "static/esp8266_webserver.ino")]
        sketch: PathBuf,
        /// Serial port (e.g., COM5 on Windows)
        #[arg(short, long, default_value = "COM5")]
        port: String,
        /// Arduino board type (default: esp8266:esp8266:nodemcuv2)
        #[arg(short, long, default_value = "esp8266:esp8266:nodemcuv2")]
        board: String,
        /// Path to arduino-cli executable
        #[arg(long)]
        arduino_cli: Option<String>,
        /// Skip SPIFFS setup
        #[arg(long)]
        skip_spiffs: bool,
        /// Skip web interface opening
        #[arg(long)]
        skip_web: bool,
        /// Skip configuration upload
        #[arg(long)]
        skip_config: bool,
        /// Path to WiFi configuration file
        #[arg(long, default_value = "static/data/wifi_config.json")]
        config_file: PathBuf,
        /// Path to HTML interface file
        #[arg(long, default_value = "static/data/arduino.html")]
        html_file: PathBuf,
    },
}

#[derive(Subcommand)]
enum ESP8266Commands {
    /// Scan network for ESP8266 devices
    Scan,
    /// Inspect specific ESP8266 device
    Inspect {
        /// IP address of the ESP8266
        #[arg(short, long)]
        ip: String,
    },
    /// Control ESP8266 LED
    Led {
        /// IP address of the ESP8266
        #[arg(short, long)]
        ip: String,
        /// Action: on, off, toggle
        #[arg(short, long)]
        action: String,
    },
    /// Control ESP8266 Relay
    Relay {
        /// IP address of the ESP8266
        #[arg(short, long)]
        ip: String,
        /// Action: on, off, toggle
        #[arg(short, long)]
        action: String,
    },
    /// Open web interface in browser
    Web {
        /// IP address of the ESP8266
        #[arg(short, long)]
        ip: String,
    },
}

fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    let deployer = ArduinoDeployer::new(true); // verbose mode
    
    match cli.command {
        Commands::UploadConfig { port, config_file, html_file, baud } => {
            println!("📁 ESP8266 Configuration Upload");
            println!("═══════════════════════════════════════");
            println!("📡 Port: {}", port);
            println!("📄 Config file: {}", config_file.display());
            if let Some(html) = &html_file {
                println!("🌐 HTML file: {}", html.display());
            }
            println!("📊 Baud rate: {}", baud);
            println!();
            
            // Create config uploader
            let uploader = ConfigUploader::new(true);
            
            // Upload configuration file
            println!("📤 Uploading configuration file...");
            uploader.upload_config_file(&port, &config_file, baud)?;
            println!("✅ Configuration file uploaded successfully");
            
            // Upload HTML file if provided
            if let Some(html) = html_file {
                println!("📤 Uploading HTML file...");
                uploader.upload_html_file(&port, &html, baud)?;
                println!("✅ HTML file uploaded successfully");
            }
            
            println!();
            println!("🎉 Upload completed! Reset the ESP8266 to load new configuration.");
            println!("═══════════════════════════════════════");
            
            Ok(())
        },
        Commands::Complete { sketch, port, board, arduino_cli, skip_spiffs, skip_web, skip_config, config_file, html_file } => {
            let work_dir = std::env::current_dir()
                .expect("Failed to get current directory");
            
            println!("🚀 Arduino ESP8266 Complete Workflow");
            println!("═══════════════════════════════════════");
            println!("📋 Sketch: {}", sketch.display());
            println!("📡 Port: {}", port);
            println!("🔧 Board: {}", board);
            println!();
            
            // Step 1: Setup SPIFFS if not skipped
            if !skip_spiffs {
                println!("📁 Step 1: Setting up SPIFFS for web interface...");
                let spiffs_manager = SpiffsManager::new(work_dir.clone(), true);
                spiffs_manager.auto_setup_if_needed()
                    .context("Failed to setup SPIFFS")?;
                println!("✅ SPIFFS setup completed");
                println!();
            }
            
            // Step 2: Deploy sketch
            println!("🔨 Step 2: Deploying sketch to ESP8266...");
            let mut deployer = ArduinoDeployer::new(true);
            if let Some(cli_path) = arduino_cli {
                deployer.set_arduino_cli_path(cli_path);
            }
            deployer.ensure_esp8266_support()?;
            deployer.deploy_sketch(sketch, &port, &board)?;
            println!("✅ Sketch deployed successfully");
            println!();
            
            // Step 3: Wait for boot and WiFi connection
            println!("⏳ Step 3: Waiting for ESP8266 to boot and connect to WiFi...");
            std::thread::sleep(std::time::Duration::from_secs(10));
            println!("✅ Boot wait completed");
            println!();
            
            // Step 4: Upload configuration files
            if !skip_config {
                println!("📤 Step 4: Uploading configuration files...");
                let config_uploader = ConfigUploader::new(true);
                
                // Upload WiFi configuration
                if config_file.exists() {
                    config_uploader.upload_config_file(&port, &config_file, 115200)
                        .context("Failed to upload WiFi configuration")?;
                    println!("✅ WiFi configuration uploaded");
                } else {
                    println!("⚠️  WiFi config not found at: {}", config_file.display());
                }
                
                // Upload HTML interface
                if html_file.exists() {
                    config_uploader.upload_html_file(&port, &html_file, 115200)
                        .context("Failed to upload HTML interface")?;
                    println!("✅ HTML interface uploaded");
                } else {
                    println!("⚠️  HTML interface not found at: {}", html_file.display());
                }
                println!();
            }
            
            // Step 5: Test and open web interface if not skipped
            if !skip_web {
                println!("🌐 Step 5: Testing ESP8266 connectivity...");
                
                // Additional wait for WiFi connection
                println!("⏳ Waiting for WiFi connection to establish...");
                std::thread::sleep(std::time::Duration::from_secs(15));
                
                let tester = ESP8266WebTester::new(true);
                let devices = tester.ultra_fast_detect_esp8266()?;
                
                if !devices.is_empty() {
                    for device in devices {
                        let detector = ESP8266Detector::new(true);
                        if let Ok(info) = detector.inspect_esp8266(&device) {
                            if info.device_name.contains("ESP8266") || info.status == "Connected" {
                                println!("✅ ESP8266 found at: {}", device);
                                info.display();
                                
                                if let Err(e) = detector.open_web_interface(&device) {
                                    println!("⚠️  Could not open web interface: {}", e);
                                    println!("🌐 Manual access: http://{}", device);
                                }
                                break;
                            }
                        }
                    }
                } else {
                    println!("⚠️  No ESP8266 devices detected on network");
                    println!("� Trying manual IP detection...");
                    
                    // Try known IP addresses
                    let known_ips = vec!["192.168.0.238", "192.168.1.238", "192.168.0.100"];
                    for ip in known_ips {
                        println!("🔍 Testing IP: {}", ip);
                        let detector = ESP8266Detector::new(true);
                        if let Ok(info) = detector.inspect_esp8266(ip) {
                            if info.status == "Connected" {
                                println!("✅ ESP8266 found at: {}", ip);
                                info.display();
                                
                                if let Err(e) = detector.open_web_interface(ip) {
                                    println!("⚠️  Could not open web interface: {}", e);
                                    println!("🌐 Manual access: http://{}", ip);
                                }
                                break;
                            }
                        }
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                    
                    println!("💡 If not found, check your WiFi credentials in wifi_config.json");
                    println!("💡 ESP8266 might still be connecting - try again in 30 seconds");
                }
            }
            
            println!();
            println!("🎉 Complete workflow finished!");
            println!("═══════════════════════════════════════");
            
            Ok(())
        },
        Commands::AutoDeploy { sketch, port, board, arduino_cli } => {
            let work_dir = std::env::current_dir()
                .expect("Failed to get current directory");
            
            let mut deployment_manager = ArduinoDeploymentManager::new(work_dir, true);
            if let Some(cli_path) = arduino_cli {
                deployment_manager.set_arduino_cli_path(cli_path);
            }
            
            // Validate environment first
            deployment_manager.validate_deployment_environment()
                .context("Environment validation failed")?;
            
            // Auto-deploy with full workflow
            deployment_manager.auto_deploy_sketch(&sketch, &port, board.as_deref())
                .context("Auto-deployment failed")?;
            
            Ok(())
        },
        Commands::SetupSpiffs { work_dir } => {
            let work_dir = work_dir.unwrap_or_else(|| std::env::current_dir()
                .expect("Failed to get current directory"));
            
            let spiffs_manager = SpiffsManager::new(work_dir, true);
            spiffs_manager.setup_spiffs_data()
                .context("Failed to setup SPIFFS data")?;
            
            println!("✅ SPIFFS setup completed!");
            println!();
            
            // Display instructions
            let commands = spiffs_manager.get_spiffs_upload_commands();
            println!("📋 Upload commands:");
            for cmd in commands {
                println!("  • {}", cmd);
            }
            
            Ok(())
        },
        Commands::TestEsp8266 { ip, auto_detect } => {
            let tester = ESP8266WebTester::new(true);
            
            if auto_detect || ip.is_none() {
                println!("⚡ Ultra-fast ESP8266 detection...");
                let devices = tester.ultra_fast_detect_esp8266()?;
                
                if devices.is_empty() {
                    println!("🔄 Trying fast detection...");
                    let devices = tester.fast_detect_esp8266()?;
                    
                    if devices.is_empty() {
                        println!("❌ No ESP8266 devices found with fast detection");
                        println!("🔄 Trying comprehensive scan...");
                        
                        let devices = tester.auto_detect_esp8266()?;
                        if devices.is_empty() {
                            println!("❌ No ESP8266 devices found on network");
                            println!("🔧 Troubleshooting tips:");
                            println!("   1. Make sure your ESP8266 is powered on and connected to WiFi");
                            println!("   2. Check that the ESP8266 is on the same network as your computer");
                            println!("   3. Verify the ESP8266 web server is running on port 80");
                            println!("   4. Try specifying the IP address directly with -i <ip_address>");
                            println!("   5. Check if firewall is blocking the connection");
                            println!("   6. Make sure the ESP8266 sketch includes the /api/status endpoint");
                            return Ok(());
                        }
                    }
                }
                
                println!("✅ Found {} ESP8266 device(s):", devices.len());
                for device in &devices {
                    println!("  🌐 Testing {}", device);
                    let result = tester.test_esp8266_server(device)?;
                    
                    if result.is_fully_functional() {
                        println!("    ✅ Fully functional");
                        tester.open_web_interface(device)?;
                    } else {
                        println!("    ⚠️  Partially functional");
                    }
                }
            } else if let Some(ip_addr) = ip {
                println!("🔍 Testing ESP8266 at {}", ip_addr);
                let result = tester.test_esp8266_server(&ip_addr)?;
                
                if result.is_fully_functional() {
                    println!("✅ ESP8266 at {} is fully functional", ip_addr);
                    tester.open_web_interface(&ip_addr)?;
                } else {
                    println!("⚠️  ESP8266 at {} has issues", ip_addr);
                    if !result.ping_success {
                        println!("❌ Basic connectivity failed");
                        println!("🔧 Troubleshooting tips:");
                        println!("   1. Verify the IP address is correct");
                        println!("   2. Check if ESP8266 is powered on");
                        println!("   3. Ensure ESP8266 is connected to the network");
                        println!("   4. Try pinging the device manually: ping {}", ip_addr);
                        println!("   5. Check if firewall is blocking the connection");
                    }
                }
            }
            
            Ok(())
        },
        Commands::List => {
            let ports = deployer.list_ports()?;
            if ports.is_empty() {
                println!("❌ No serial ports found");
            } else {
                println!("📡 Available serial ports:");
                for port in ports {
                    println!("  • {}", port);
                }
            }
            Ok(())
        },
        Commands::Deploy { sketch, port, board, arduino_cli } => {
            let mut deployer = ArduinoDeployer::new(true);
            if let Some(cli_path) = arduino_cli {
                deployer.set_arduino_cli_path(cli_path);
            }
            deployer.deploy_sketch(sketch, &port, &board)
        },
        Commands::DeployWeb { sketch, port, board, arduino_cli, skip_spiffs } => {
            let work_dir = std::env::current_dir()
                .expect("Failed to get current directory");
            
            // Setup SPIFFS if not skipped
            if !skip_spiffs {
                println!("🔧 Setting up SPIFFS for web interface...");
                let spiffs_manager = SpiffsManager::new(work_dir.clone(), true);
                spiffs_manager.auto_setup_if_needed()
                    .context("Failed to setup SPIFFS")?;
                println!();
            }
            
            let mut deployer = ArduinoDeployer::new(true);
            if let Some(cli_path) = arduino_cli {
                deployer.set_arduino_cli_path(cli_path);
            }
            
            // Auto-install ESP8266 support if needed
            deployer.ensure_esp8266_support()?;
            
            // Deploy the sketch
            println!("🚀 Deploying ESP8266 sketch...");
            deployer.deploy_sketch(sketch, &port, &board)?;
            
            // Wait a bit for ESP8266 to boot
            println!("⏳ Waiting for ESP8266 to boot...");
            std::thread::sleep(std::time::Duration::from_secs(3));
            
            // Ultra-fast scan for ESP8266 (< 5 seconds total)
            let tester = ESP8266WebTester::new(true);
            println!("⚡ Ultra-fast ESP8266 detection...");
            
            let devices = tester.ultra_fast_detect_esp8266()?;
            if !devices.is_empty() {
                for device in devices {
                    let detector = ESP8266Detector::new(true);
                    if let Ok(info) = detector.inspect_esp8266(&device) {
                        if info.device_name.contains("ESP8266") || info.status == "Connected" {
                            println!("✅ Found ESP8266 at: {}", device);
                            info.display();
                            
                            // Open web interface
                            if let Err(e) = detector.open_web_interface(&device) {
                                println!("⚠️  Could not open web interface: {}", e);
                                println!("🌐 You can manually open: http://{}", device);
                            }
                            
                            return Ok(());
                        }
                    }
                }
            }
            
            // Quick fallback to fast detection if ultra-fast fails
            println!("🔄 Trying fast detection...");
            let devices = tester.fast_detect_esp8266()?;
            for device in devices {
                let detector = ESP8266Detector::new(true);
                if let Ok(info) = detector.inspect_esp8266(&device) {
                    if info.device_name.contains("ESP8266") || info.status == "Connected" {
                        println!("✅ Found ESP8266 at: {}", device);
                        info.display();
                        
                        // Open web interface
                        if let Err(e) = detector.open_web_interface(&device) {
                            println!("⚠️  Could not open web interface: {}", e);
                            println!("🌐 You can manually open: http://{}", device);
                        }
                        
                        return Ok(());
                    }
                }
            }
            
            println!("❌ No ESP8266 found. Please check your device and network connection.");
            Ok(())
        },
        Commands::NetworkDiag { ip } => {
            let tester = ESP8266WebTester::new(true);
            
            // Show network diagnostics
            tester.print_network_diagnostics()?;
            
            // If IP is provided, test it specifically
            if let Some(ip_addr) = ip {
                println!();
                tester.test_ip_detailed(&ip_addr)?;
            } else {
                println!();
                println!("💡 Use -i <ip_address> to test a specific IP address");
            }
            
            Ok(())
        },
        Commands::Monitor { port, baud } => {
            deployer.monitor_serial(&port, baud)
        },
        Commands::Example { output } => {
            create_example_sketch(output)?;
            Ok(())
        },
        Commands::Boards => {
            println!("🔧 Supported Arduino boards:");
            let boards = BoardInfo::list_supported_boards();
            for board in boards {
                println!("  • {} - {} ({})", board.name, board.description, board.fqbn);
            }
            Ok(())
        },
        Commands::Check => {
            check_arduino_cli()
        },
        Commands::AutoDetect => {
            let deployer = ArduinoDeployer::new(true);
            let ports = deployer.list_ports()?;
            if let Some(arduino_port) = ports.iter().find(|p| p.contains("Arduino") || p.contains("USB") || p.contains("COM")) {
                println!("✅ Arduino detected on port: {}", arduino_port);
            } else if !ports.is_empty() {
                println!("🔍 Available ports (Arduino not specifically detected):");
                for port in ports {
                    println!("  • {}", port);
                }
            } else {
                println!("❌ No ports found");
            }
            Ok(())
        },
        Commands::Esp8266 { esp_command } => {
            let detector = ESP8266Detector::new(true);
            
            match esp_command {
                ESP8266Commands::Scan => {
                    let devices = detector.scan_network()?;
                    if devices.is_empty() {
                        println!("❌ No ESP8266 devices found on network");
                    } else {
                        println!("✅ Found {} ESP8266 device(s):", devices.len());
                        for device in devices {
                            println!("  🌐 {}", device);
                            if let Ok(info) = detector.inspect_esp8266(&device) {
                                println!("     📱 Device: {}", info.device_name);
                                println!("     🔌 Status: {}", info.status);
                                println!("     📡 Web: http://{}", device);
                            }
                        }
                    }
                    Ok(())
                },
                ESP8266Commands::Inspect { ip } => {
                    let info = detector.inspect_esp8266(&ip)?;
                    info.display();
                    Ok(())
                },
                ESP8266Commands::Led { ip, action } => {
                    let result = detector.control_led(&ip, &action)?;
                    println!("💡 LED Control Result: {}", result);
                    Ok(())
                },
                ESP8266Commands::Relay { ip, action } => {
                    let result = detector.control_relay(&ip, &action)?;
                    println!("🔌 Relay Control Result: {}", result);
                    Ok(())
                },
                ESP8266Commands::Web { ip } => {
                    detector.open_web_interface(&ip)?;
                    println!("🌐 Opening web interface at http://{}", ip);
                    Ok(())
                },
            }
        },
    }
}


