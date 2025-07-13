# Arduino Deployer

Simple Rust application to deploy Arduino sketches via USB.

## Features

- 📡 List available serial ports
- 🚀 Compile and upload Arduino sketches
- 📺 Monitor serial output
- 🔧 Support for common Arduino boards (Uno, Nano, Mega, Leonardo)
- 📄 Sketch validation
- 🖥️ Cross-platform (Windows, Linux, macOS)
- 🔧 **Uses local Arduino CLI** (no system installation required)

## Setup

1. **Place Arduino CLI** in the `cli/` directory:
   - **Windows**: Download and extract `arduino-cli.exe` to `cli/arduino-cli.exe`
   - **Linux/macOS**: Download and extract `arduino-cli` to `cli/arduino-cli`
   
   Download from: https://github.com/arduino/arduino-cli/releases/latest

2. **Run the script**:
   - **Windows**: `run_arduino.bat`
   - **Linux/macOS**: `./run_arduino.sh`

That's it! The script will automatically configure Arduino CLI and present a menu.

## Quick Start

**Windows:**
```cmd
run_arduino.bat
```

**Linux/macOS:**
```bash
./run_arduino.sh
```

The script will:
1. **Use local Arduino CLI** from `cli/` directory (or system-wide if available)
2. **Auto-configure Arduino CLI** (download AVR core, etc.)
3. **Build the project** 
4. **Present a simple menu** to choose what you want to do

## Manual Usage

If you prefer command line:

```bash
# List available ports
./target/release/arduino-deploy list

# Deploy a sketch
./target/release/arduino-deploy deploy --sketch examples/blink.ino --port COM3 --board uno

# Monitor serial output
./target/release/arduino-deploy monitor --port COM3 --baud 9600

# Create example sketch
./target/release/arduino-deploy example

# Show supported boards
./target/release/arduino-deploy boards
```

## Supported Boards

- `uno` - Arduino Uno R3 (ATmega328P)
- `nano` - Arduino Nano (ATmega328P)
- `mega` - Arduino Mega 2560 (ATmega2560)
- `leonardo` - Arduino Leonardo (ATmega32u4)

## Common Port Names

- **Windows**: `COM3`, `COM4`, etc.
- **Linux**: `/dev/ttyUSB0`, `/dev/ttyACM0`, etc.
- **macOS**: `/dev/cu.usbserial-*`, `/dev/cu.usbmodem-*`, etc.

## That's it!

This is intentionally simple. Just place Arduino CLI in the `cli/` folder and run the script!
