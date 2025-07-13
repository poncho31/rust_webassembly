use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;
use arduino_deployer::{ArduinoDeployer, BoardInfo, check_arduino_cli, create_example_sketch, ESP8266Detector};

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
    /// Deploy ESP8266 with auto-setup and web interface opening
    DeployWeb {
        /// Path to the sketch file (.ino)
        #[arg(short, long)]
        sketch: PathBuf,
        /// Serial port (e.g., COM5 on Windows)
        #[arg(short, long)]
        port: String,
        /// Arduino board type (default: esp8266:esp8266:nodemcuv2)
        #[arg(short, long, default_value = "esp8266:esp8266:nodemcuv2")]
        board: String,
        /// Path to arduino-cli executable
        #[arg(long)]
        arduino_cli: Option<String>,
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
        #[arg(short, long, default_value = "examples/blink.ino")]
        output: PathBuf,
    },
    /// Show supported boards
    Boards,
    /// Check arduino-cli installation
    Check,
    /// Auto-detect Arduino port
    AutoDetect,
    /// ESP8266 network scanner and web interface
    Esp8266 {
        #[command(subcommand)]
        esp_command: ESP8266Commands,
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
        Commands::DeployWeb { sketch, port, board, arduino_cli } => {
            let mut deployer = ArduinoDeployer::new(true);
            if let Some(cli_path) = arduino_cli {
                deployer.set_arduino_cli_path(cli_path);
            }
            
            // Auto-install ESP8266 support if needed
            deployer.ensure_esp8266_support()?;
            
            // Deploy the sketch
            deployer.deploy_sketch(sketch, &port, &board)?;
            
            // Wait a bit for ESP8266 to boot
            println!("⏳ Waiting for ESP8266 to boot...");
            std::thread::sleep(std::time::Duration::from_secs(5));
            
            // Scan for ESP8266 and open web interface
            let detector = ESP8266Detector::new(true);
            println!("🔍 Scanning for ESP8266 devices...");
            
            let devices = detector.scan_network()?;
            for device in devices {
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


