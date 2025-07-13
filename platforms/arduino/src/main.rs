use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;
use arduino_deployer::{ArduinoDeployer, BoardInfo, check_arduino_cli, create_example_sketch};

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
    }
}


