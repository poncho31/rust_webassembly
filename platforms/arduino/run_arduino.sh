#!/bin/bash

echo "Arduino Deployer - Simple Arduino Code Deployment Tool"
echo "======================================================="

# Function to run tests
run_tests() {
    echo "Running Arduino Deployer tests..."
    echo ""
    
    echo "[TEST 1] Testing help command..."
    $ARDUINO_EXE --help || { echo "❌ Test failed!"; return 1; }
    
    echo "[TEST 2] Testing boards command..."
    $ARDUINO_EXE boards || { echo "❌ Test failed!"; return 1; }
    
    echo "[TEST 3] Testing list command..."
    $ARDUINO_EXE list || { echo "❌ Test failed!"; return 1; }
    
    echo "[TEST 4] Testing example creation..."
    $ARDUINO_EXE example --output "examples/test_blink.ino" || { echo "❌ Test failed!"; return 1; }
    [ -f "examples/test_blink.ino" ] || { echo "❌ Test failed!"; return 1; }
    
    echo "[TEST 5] Testing arduino-cli check..."
    $ARDUINO_EXE check
    if [ $? -ne 0 ]; then
        echo "Arduino CLI not found - some tests skipped"
    else
        echo "Arduino CLI OK"
    fi
    
    # Cleanup
    rm -f "examples/test_blink.ino"
    
    echo ""
    echo "✅ All tests passed!"
    echo ""
    return 0
}

# Check if arduino-cli is installed
LOCAL_ARDUINO_CLI="$(dirname "$0")/cli/arduino-cli"
if [ -f "$LOCAL_ARDUINO_CLI" ]; then
    echo "✅ Using local Arduino CLI: $LOCAL_ARDUINO_CLI"
    ARDUINO_CLI="$LOCAL_ARDUINO_CLI"
else
    if ! command -v arduino-cli &> /dev/null; then
        echo ""
        echo "❌ Arduino CLI not found!"
        echo "Please place arduino-cli in: $(dirname "$0")/cli/arduino-cli"
        echo "Or install it system-wide from: https://arduino.github.io/arduino-cli/installation/"
        echo ""
        echo "Quick install:"
        echo "  Linux: curl -fsSL https://raw.githubusercontent.com/arduino/arduino-cli/master/install.sh | BINDIR=/usr/local/bin sh"
        echo "  macOS: brew install arduino-cli"
        echo ""
        exit 1
    else
        echo "✅ Using system Arduino CLI"
        ARDUINO_CLI="arduino-cli"
    fi
fi

# Initialize Arduino CLI if needed
echo "Checking Arduino CLI configuration..."
$ARDUINO_CLI config init >/dev/null 2>&1
$ARDUINO_CLI core update-index >/dev/null 2>&1
$ARDUINO_CLI core install arduino:avr >/dev/null 2>&1
echo "✅ Arduino CLI ready!"

# Build the project
echo "Building Arduino deployer..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "ERROR: Failed to build project"
    exit 1
fi

# Set executable path
ARDUINO_EXE="../../target/release/arduino-deploy"

# Run the application
echo ""
echo "Available commands:"
echo "  list                   - List available serial ports"
echo "  deploy                 - Deploy a sketch to Arduino"
echo "  monitor                - Monitor serial output"
echo "  example                - Create example sketch"
echo "  test                   - Test the application"
echo "  boards                 - Show supported boards"
echo ""

read -p "Choose command (list/deploy/monitor/example/test/boards): " choice

case "$choice" in
    "list")
        $ARDUINO_EXE list
        ;;
    "deploy")
        read -p "Enter sketch path (.ino file): " sketch
        read -p "Enter port (e.g., /dev/ttyUSB0): " port
        read -p "Enter board type (uno/nano/mega/leonardo): " board
        $ARDUINO_EXE deploy --sketch "$sketch" --port "$port" --board "$board"
        ;;
    "monitor")
        read -p "Enter port to monitor: " port
        read -p "Enter baud rate (default 9600): " baud
        if [ -z "$baud" ]; then
            baud="9600"
        fi
        $ARDUINO_EXE monitor --port "$port" --baud "$baud"
        ;;
    "example")
        $ARDUINO_EXE example
        echo "Example sketch created at: examples/blink.ino"
        ;;
    "test")
        run_tests
        ;;
    "boards")
        $ARDUINO_EXE boards
        ;;
    *)
        echo "Invalid choice. Please try again."
        ;;
esac
