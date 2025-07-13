// ESP8266 Complete Server - No External Libraries
// Complete WiFi + Web Server + API + Sensors + Controls for NodeMCU/ESP8266
// Uses only built-in ESP8266 libraries - no external dependencies!

#include <ESP8266WiFi.h>
#include <ESP8266WebServer.h>
#include <ESP8266mDNS.h>
#include <EEPROM.h>
#include <WiFiUdp.h>
#include <Ticker.h>
#include <FS.h>

// ==================== CONFIGURATION ====================
// Configuration structure
struct Config {
  String wifi_ssid;
  String wifi_password;
  String device_name;
  String device_version;
  int server_port;
  int sensor_interval;
  bool auto_relay;
  int led_pin;
  int relay_pin;
  int button_pin;
  int sensor_pin;
  int pwm_pin;
  int analog_pin;
} config;

// Default configuration (fallback)
void loadDefaultConfig() {
  config.wifi_ssid = "test";
  config.wifi_password = "test";
  config.device_name = "ESP8266-Complete";
  config.device_version = "1.0.0";
  config.server_port = 80;
  config.sensor_interval = 5000;
  config.auto_relay = false;
  config.led_pin = LED_BUILTIN;
  config.relay_pin = 12;
  config.button_pin = 0;
  config.sensor_pin = 14;
  config.pwm_pin = 13;
  config.analog_pin = A0;
}

// Load configuration from SPIFFS
bool loadConfig() {
  Serial.println("📋 Loading configuration from SPIFFS...");
  
  if (!SPIFFS.begin()) {
    Serial.println("⚠️  SPIFFS not available, formatting...");
    if (SPIFFS.format()) {
      Serial.println("✅ SPIFFS formatted successfully");
      if (SPIFFS.begin()) {
        Serial.println("✅ SPIFFS mounted after format");
      } else {
        Serial.println("❌ SPIFFS mount failed even after format");
        loadDefaultConfig();
        return false;
      }
    } else {
      Serial.println("❌ SPIFFS format failed");
      loadDefaultConfig();
      return false;
    }
  }
  
  if (!SPIFFS.exists("/wifi_config.json")) {
    Serial.println("⚠️  Configuration file not found, creating default...");
    createDefaultConfigFile();
    Serial.println("💡 Please update wifi_config.json with your credentials");
    loadDefaultConfig();
    return false;
  }
  
  File configFile = SPIFFS.open("/wifi_config.json", "r");
  if (!configFile) {
    Serial.println("⚠️  Failed to open config file");
    loadDefaultConfig();
    return false;
  }
  
  String configData = configFile.readString();
  configFile.close();
  
  Serial.println("📄 Config file content:");
  Serial.println(configData);
  
  // Parse JSON manually (simple parsing for ESP8266)
  bool parseResult = parseConfig(configData);
  
  if (!parseResult) {
    Serial.println("⚠️  Failed to parse config file");
    loadDefaultConfig();
    return false;
  }
  
  return true;
}

// Create default configuration file in SPIFFS
void createDefaultConfigFile() {
  Serial.println("📝 Creating default configuration file...");
  
  String defaultConfig = R"({
  "wifi": {
    "ssid": "YOUR_WIFI_SSID",
    "password": "YOUR_WIFI_PASSWORD"
  },
  "device": {
    "name": "ESP8266-Complete",
    "version": "1.0.0",
    "port": 80
  },
  "sensors": {
    "interval": 5000,
    "auto_relay": false
  },
  "pins": {
    "led": "LED_BUILTIN",
    "relay": 12,
    "button": 0,
    "sensor": 14,
    "pwm": 13,
    "analog": "A0"
  }
})";
  
  File configFile = SPIFFS.open("/wifi_config.json", "w");
  if (configFile) {
    configFile.print(defaultConfig);
    configFile.close();
    Serial.println("✅ Default configuration file created");
    Serial.println("📝 Edit /wifi_config.json with your WiFi credentials");
  } else {
    Serial.println("❌ Failed to create configuration file");
  }
}

// Simple JSON parser for configuration
bool parseConfig(String json) {
  // Remove whitespace and newlines
  json.trim();
  json.replace("\n", "");
  json.replace("\r", "");
  json.replace(" ", "");
  
  // Extract WiFi configuration
  int ssidStart = json.indexOf("\"ssid\":\"") + 8;
  int ssidEnd = json.indexOf("\"", ssidStart);
  if (ssidStart > 7 && ssidEnd > ssidStart) {
    config.wifi_ssid = json.substring(ssidStart, ssidEnd);
  }
  
  int passStart = json.indexOf("\"password\":\"") + 12;
  int passEnd = json.indexOf("\"", passStart);
  if (passStart > 11 && passEnd > passStart) {
    config.wifi_password = json.substring(passStart, passEnd);
  }
  
  // Extract device configuration
  int nameStart = json.indexOf("\"name\":\"") + 8;
  int nameEnd = json.indexOf("\"", nameStart);
  if (nameStart > 7 && nameEnd > nameStart) {
    config.device_name = json.substring(nameStart, nameEnd);
  }
  
  int versionStart = json.indexOf("\"version\":\"") + 11;
  int versionEnd = json.indexOf("\"", versionStart);
  if (versionStart > 10 && versionEnd > versionStart) {
    config.device_version = json.substring(versionStart, versionEnd);
  }
  
  // Extract sensor configuration
  int intervalStart = json.indexOf("\"interval\":") + 11;
  int intervalEnd = json.indexOf(",", intervalStart);
  if (intervalEnd == -1) intervalEnd = json.indexOf("}", intervalStart);
  if (intervalStart > 10 && intervalEnd > intervalStart) {
    config.sensor_interval = json.substring(intervalStart, intervalEnd).toInt();
  }
  
  // Extract pins configuration
  int relayStart = json.indexOf("\"relay\":") + 8;
  int relayEnd = json.indexOf(",", relayStart);
  if (relayEnd == -1) relayEnd = json.indexOf("}", relayStart);
  if (relayStart > 7 && relayEnd > relayStart) {
    config.relay_pin = json.substring(relayStart, relayEnd).toInt();
  }
  
  // Check if WiFi credentials are properly configured
  if (config.wifi_ssid.length() == 0 || config.wifi_password.length() == 0) {
    Serial.println("⚠️  WiFi credentials are empty in wifi_config.json");
    Serial.println("   Please edit wifi_config.json with your network credentials");
    return false;
  }
  
  // Allow any SSID/password - don't check for placeholder values
  Serial.println("✅ WiFi credentials loaded from config file");
  Serial.print("   SSID: ");
  Serial.println(config.wifi_ssid);
  
  return true;
}

// Device configuration (will be loaded from config)
const char* deviceName = "ESP8266-Complete";  // Will be updated from config
const char* deviceVersion = "1.0.0";          // Will be updated from config

// Pin definitions (will be loaded from config)
int ledPin = LED_BUILTIN;
int analogPin = A0;
int relayPin = 12;     // D6 on NodeMCU
int buttonPin = 0;     // D3 on NodeMCU (Flash button)
int sensorPin = 14;    // D5 on NodeMCU
int pwmPin = 13;       // D7 on NodeMCU

// ==================== GLOBAL VARIABLES ====================
ESP8266WebServer server(80);
WiFiUDP udp;
Ticker systemTicker;

// Device state
bool ledState = false;
bool relayState = false;
bool buttonState = false;
bool lastButtonState = false;
int analogValue = 0;
int sensorValue = 0;
int pwmValue = 0;
unsigned long uptime = 0;
unsigned long lastSensorRead = 0;
unsigned long buttonPressTime = 0;

// WiFi and system stats
int wifiReconnectCount = 0;
unsigned long lastWifiCheck = 0;
String lastError = "";

// Settings (stored in EEPROM)
struct Settings {
  char deviceName[32];
  int sensorInterval;
  bool autoRelay;
  int pwmDefault;
  char signature[8]; // To validate EEPROM data
} settings;

// ==================== SETUP ====================
void setup() {
  Serial.begin(115200);
  Serial.println();
  Serial.println("==============================================");
  Serial.println("🚀 ESP8266 Complete Server Starting...");
  Serial.println("==============================================");
  
  // Load configuration first
  Serial.println("📋 Loading configuration...");
  if (loadConfig()) {
    Serial.println("✅ Configuration loaded from wifi_config.json");
  } else {
    Serial.println("⚠️  Using default configuration");
  }
  
  // Update global variables from config
  deviceName = config.device_name.c_str();
  deviceVersion = config.device_version.c_str();
  ledPin = config.led_pin;
  analogPin = config.analog_pin;
  relayPin = config.relay_pin;
  buttonPin = config.button_pin;
  sensorPin = config.sensor_pin;
  pwmPin = config.pwm_pin;
  
  // Initialize EEPROM and load settings
  EEPROM.begin(512);
  loadSettings();
  
  // Initialize pins
  initializePins();
  
  // Connect to WiFi
  connectToWiFi();
  
  // Initialize services
  initializeServices();
  
  // Setup web server routes
  setupWebServer();
  
  // Start system ticker (every second)
  systemTicker.attach(1.0, systemTickerCallback);
  
  Serial.println("✅ System ready!");
  Serial.println("==============================================");
  printSystemInfo();
  Serial.println("==============================================");
}

// ==================== MAIN LOOP ====================
void loop() {
  // Handle web server
  server.handleClient();
  
  // Handle mDNS
  MDNS.update();
  
  // Handle serial uploads
  handleSerialUpload();
  
  // Read sensors
  readSensors();
  
  // Handle button
  handleButton();
  
  // Check WiFi connection
  checkWiFiConnection();
  
  // Auto relay logic
  if (settings.autoRelay) {
    autoRelayLogic();
  }
  
  delay(10);
}

// ==================== INITIALIZATION ====================
void initializePins() {
  pinMode(ledPin, OUTPUT);
  pinMode(relayPin, OUTPUT);
  pinMode(buttonPin, INPUT_PULLUP);
  pinMode(sensorPin, INPUT);
  pinMode(pwmPin, OUTPUT);
  
  // Set initial states
  digitalWrite(ledPin, HIGH);  // Turn off LED (inverted logic)
  digitalWrite(relayPin, LOW); // Turn off relay
  analogWrite(pwmPin, settings.pwmDefault);
  
  Serial.println("📌 Pins initialized");
}

void connectToWiFi() {
  WiFi.mode(WIFI_STA);
  
  Serial.print("🔄 Connecting to WiFi: ");
  Serial.println(config.wifi_ssid);
  Serial.print("� Password length: ");
  Serial.println(config.wifi_password.length());
  
  WiFi.begin(config.wifi_ssid.c_str(), config.wifi_password.c_str());
  
  int attempts = 0;
  
  while (WiFi.status() != WL_CONNECTED && attempts < 30) {
    delay(500);
    Serial.print(".");
    attempts++;
    
    // Blink LED while connecting
    digitalWrite(ledPin, !digitalRead(ledPin));
  }
  
  if (WiFi.status() == WL_CONNECTED) {
    digitalWrite(ledPin, LOW); // Turn on LED when connected
    Serial.println();
    Serial.println("✅ WiFi connected!");
    Serial.println("══════════════════════════════════════════════");
    Serial.print("🌐 WEB SERVER READY: http://");
    Serial.println(WiFi.localIP());
    Serial.println("══════════════════════════════════════════════");
    Serial.print("📡 IP address: ");
    Serial.println(WiFi.localIP());
    Serial.print("📶 Signal strength: ");
    Serial.print(WiFi.RSSI());
    Serial.println(" dBm");
    Serial.print("🌐 Gateway: ");
    Serial.println(WiFi.gatewayIP());
    Serial.print("🔗 Subnet mask: ");
    Serial.println(WiFi.subnetMask());
    Serial.print("🔗 DNS server: ");
    Serial.println(WiFi.dnsIP());
    Serial.print("📶 Channel: ");
    Serial.println(WiFi.channel());
    Serial.print("🏷️  MAC address: ");
    Serial.println(WiFi.macAddress());
    Serial.println("══════════════════════════════════════════════");
    Serial.println("📋 Access the web interface at:");
    Serial.print("   http://");
    Serial.println(WiFi.localIP());
    Serial.println("══════════════════════════════════════════════");
  } else {
    Serial.println();
    Serial.println("❌ WiFi connection failed!");
    Serial.println("🔧 Troubleshooting tips:");
    Serial.println("   1. Check wifi_config.json for correct SSID and password");
    Serial.println("   2. Ensure WiFi network is 2.4GHz");
    Serial.println("   3. Check if MAC filtering is enabled");
    Serial.println("   4. Try moving closer to the router");
    Serial.println("   5. Check if network is hidden");
    Serial.print("   Current SSID: ");
    Serial.println(config.wifi_ssid);
    lastError = "WiFi connection failed";
  }
}

void initializeServices() {
  // Initialize SPIFFS with better error handling
  Serial.println("📁 Initializing SPIFFS...");
  if (!SPIFFS.begin()) {
    Serial.println("⚠️  SPIFFS Mount Failed - attempting to format...");
    if (SPIFFS.format()) {
      Serial.println("✅ SPIFFS formatted successfully");
      if (SPIFFS.begin()) {
        Serial.println("✅ SPIFFS mounted after format");
      } else {
        Serial.println("❌ SPIFFS mount failed even after format");
      }
    } else {
      Serial.println("❌ SPIFFS format failed - HTML file will not be available");
    }
  } else {
    Serial.println("✅ SPIFFS initialized successfully");
    
    // Check for arduino.html file
    if (SPIFFS.exists("/arduino.html")) {
      File file = SPIFFS.open("/arduino.html", "r");
      if (file) {
        Serial.println("✅ arduino.html found and accessible (" + String(file.size()) + " bytes)");
        file.close();
      } else {
        Serial.println("❌ arduino.html found but not accessible");
      }
    } else {
      Serial.println("⚠️  arduino.html not found in SPIFFS");
      Serial.println("💡 Upload arduino.html using 'ESP8266 Sketch Data Upload' tool");
    }
  }
  
  // Start mDNS
  if (MDNS.begin(deviceName)) {
    Serial.println("🌐 mDNS responder started");
    Serial.print("🔗 Access via: http://");
    Serial.print(deviceName);
    Serial.println(".local");
    
    // Add service discovery
    MDNS.addService("http", "tcp", 80);
    MDNS.addService("arduino", "tcp", 80);
  } else {
    Serial.println("❌ mDNS failed to start");
  }
}

// ==================== SETTINGS MANAGEMENT ====================
void loadSettings() {
  EEPROM.get(0, settings);
  
  // Check if settings are valid using signature
  if (strcmp(settings.signature, "ESP8266") != 0) {
    // Load default settings
    strcpy(settings.deviceName, deviceName);
    settings.sensorInterval = 5000;
    settings.autoRelay = false;
    settings.pwmDefault = 128;
    strcpy(settings.signature, "ESP8266");
    
    saveSettings();
    Serial.println("📝 Default settings loaded");
  } else {
    Serial.println("📝 Settings loaded from EEPROM");
  }
}

void saveSettings() {
  EEPROM.put(0, settings);
  EEPROM.commit();
  Serial.println("💾 Settings saved to EEPROM");
}

// ==================== SENSOR READING ====================
void readSensors() {
  if (millis() - lastSensorRead > settings.sensorInterval) {
    analogValue = analogRead(analogPin);
    sensorValue = digitalRead(sensorPin);
    lastSensorRead = millis();
  }
}

void handleButton() {
  buttonState = !digitalRead(buttonPin); // Inverted logic
  
  // Detect button press
  if (buttonState && !lastButtonState) {
    buttonPressTime = millis();
  }
  
  // Detect button release
  if (!buttonState && lastButtonState) {
    unsigned long pressDuration = millis() - buttonPressTime;
    
    if (pressDuration > 50 && pressDuration < 2000) {
      // Short press - toggle LED
      toggleLED();
      Serial.println("🔘 Button pressed - LED toggled");
    } else if (pressDuration >= 2000) {
      // Long press - toggle relay
      toggleRelay();
      Serial.println("🔘 Button long pressed - Relay toggled");
    }
  }
  
  lastButtonState = buttonState;
}

void autoRelayLogic() {
  // Example: Turn on relay if analog value > 512
  if (analogValue > 512 && !relayState) {
    setRelay(true);
    Serial.println("🔄 Auto relay ON (analog > 512)");
  } else if (analogValue <= 512 && relayState) {
    setRelay(false);
    Serial.println("🔄 Auto relay OFF (analog <= 512)");
  }
}

void checkWiFiConnection() {
  if (millis() - lastWifiCheck > 30000) { // Check every 30 seconds
    if (WiFi.status() != WL_CONNECTED) {
      Serial.println("❌ WiFi disconnected - attempting reconnect");
      WiFi.begin(config.wifi_ssid.c_str(), config.wifi_password.c_str());
      wifiReconnectCount++;
      lastError = "WiFi reconnection attempt";
    }
    lastWifiCheck = millis();
  }
}

// ==================== CONTROL FUNCTIONS ====================
void toggleLED() {
  ledState = !ledState;
  digitalWrite(ledPin, ledState ? LOW : HIGH); // Inverted logic
}

void setLED(bool state) {
  ledState = state;
  digitalWrite(ledPin, ledState ? LOW : HIGH); // Inverted logic
}

void toggleRelay() {
  relayState = !relayState;
  digitalWrite(relayPin, relayState ? HIGH : LOW);
}

void setRelay(bool state) {
  relayState = state;
  digitalWrite(relayPin, relayState ? HIGH : LOW);
}

void setPWM(int value) {
  pwmValue = constrain(value, 0, 255);
  analogWrite(pwmPin, pwmValue);
}

// ==================== SYSTEM CALLBACKS ====================
void systemTickerCallback() {
  uptime++;
}

void printSystemInfo() {
  Serial.println("📊 System Information:");
  Serial.print("  Device: ");
  Serial.println(settings.deviceName);
  Serial.print("  Version: ");
  Serial.println(deviceVersion);
  Serial.print("  Chip ID: ");
  Serial.println(ESP.getChipId(), HEX);
  Serial.print("  Flash Size: ");
  Serial.print(ESP.getFlashChipSize());
  Serial.println(" bytes");
  Serial.print("  Free Heap: ");
  Serial.print(ESP.getFreeHeap());
  Serial.println(" bytes");
  
  if (WiFi.status() == WL_CONNECTED) {
    Serial.println("📡 Network Information:");
    Serial.print("  SSID: ");
    Serial.println(WiFi.SSID());
    Serial.print("  IP: ");
    Serial.println(WiFi.localIP());
    Serial.print("  MAC: ");
    Serial.println(WiFi.macAddress());
    Serial.print("  Gateway: ");
    Serial.println(WiFi.gatewayIP());
    Serial.print("  DNS: ");
    Serial.println(WiFi.dnsIP());
  }
}

// ==================== UTILITY FUNCTIONS ====================
String formatUptime(unsigned long seconds) {
  unsigned long days = seconds / 86400;
  unsigned long hours = (seconds % 86400) / 3600;
  unsigned long minutes = (seconds % 3600) / 60;
  unsigned long secs = seconds % 60;
  
  String result = "";
  if (days > 0) {
    result += String(days) + "d ";
  }
  if (hours > 0) {
    result += String(hours) + "h ";
  }
  if (minutes > 0) {
    result += String(minutes) + "m ";
  }
  if (days == 0) {
    result += String(secs) + "s";
  }
  
  return result.length() > 0 ? result : "0s";
}

String formatBytes(unsigned long bytes) {
  if (bytes >= 1024 * 1024) {
    return String(bytes / (1024.0 * 1024.0), 1) + " MB";
  } else if (bytes >= 1024) {
    return String(bytes / 1024.0, 1) + " KB";
  } else {
    return String(bytes) + " B";
  }
}

String getSignalQuality(int rssi) {
  if (rssi >= -50) return "Excellent";
  if (rssi >= -60) return "Good";
  if (rssi >= -70) return "Fair";
  return "Poor";
}

// ==================== JSON HELPERS (Manual) ====================
String createJsonResponse(String status, String message = "") {
  String json = "{";
  json += "\"status\":\"" + status + "\"";
  if (message.length() > 0) {
    json += ",\"message\":\"" + message + "\"";
  }
  json += "}";
  return json;
}

String createStatusJson() {
  String json = "{";
  json += "\"status\":\"ok\",";
  json += "\"device_name\":\"" + String(settings.deviceName) + "\",";
  json += "\"version\":\"" + String(deviceVersion) + "\",";
  json += "\"uptime\":" + String(uptime) + ",";
  json += "\"free_heap\":" + String(ESP.getFreeHeap()) + ",";
  json += "\"chip_id\":" + String(ESP.getChipId()) + ",";
  json += "\"flash_size\":" + String(ESP.getFlashChipSize()) + ",";
  json += "\"led_state\":" + String(ledState ? "true" : "false") + ",";
  json += "\"relay_state\":" + String(relayState ? "true" : "false") + ",";
  json += "\"button_state\":" + String(buttonState ? "true" : "false") + ",";
  json += "\"analog_value\":" + String(analogValue) + ",";
  json += "\"sensor_value\":" + String(sensorValue) + ",";
  json += "\"pwm_value\":" + String(pwmValue) + ",";
  json += "\"wifi_rssi\":" + String(WiFi.RSSI()) + ",";
  json += "\"wifi_reconnects\":" + String(wifiReconnectCount) + ",";
  json += "\"last_error\":\"" + lastError + "\"";
  json += "}";
  return json;
}

String createControlJson() {
  String json = "{";
  json += "\"led_state\":" + String(ledState ? "true" : "false") + ",";
  json += "\"relay_state\":" + String(relayState ? "true" : "false") + ",";
  json += "\"button_state\":" + String(buttonState ? "true" : "false") + ",";
  json += "\"analog_value\":" + String(analogValue) + ",";
  json += "\"sensor_value\":" + String(sensorValue) + ",";
  json += "\"pwm_value\":" + String(pwmValue);
  json += "}";
  return json;
}

// ==================== WEB SERVER SETUP ====================
void setupWebServer() {
  // Enable CORS
  server.enableCORS(true);
  
  // Main dashboard
  server.on("/", HTTP_GET, handleRoot);
  
  // API endpoints
  server.on("/api/status", HTTP_GET, handleApiStatus);
  server.on("/api/control", HTTP_GET, handleApiControl);
  server.on("/api/control", HTTP_POST, handleApiControlPost);
  server.on("/api/system", HTTP_GET, handleApiSystem);
  server.on("/api/wifi", HTTP_GET, handleApiWifi);
  server.on("/api/reset", HTTP_POST, handleApiReset);
  
  // Simple control endpoints
  server.on("/led/on", HTTP_GET, []() { setLED(true); server.send(200, "text/plain", "LED ON"); });
  server.on("/led/off", HTTP_GET, []() { setLED(false); server.send(200, "text/plain", "LED OFF"); });
  server.on("/led/toggle", HTTP_GET, []() { toggleLED(); server.send(200, "text/plain", "LED toggled"); });
  
  server.on("/relay/on", HTTP_GET, []() { setRelay(true); server.send(200, "text/plain", "Relay ON"); });
  server.on("/relay/off", HTTP_GET, []() { setRelay(false); server.send(200, "text/plain", "Relay OFF"); });
  server.on("/relay/toggle", HTTP_GET, []() { toggleRelay(); server.send(200, "text/plain", "Relay toggled"); });
  
  // 404 handler
  server.onNotFound(handle404);
  
  // Start server
  server.begin();
  Serial.println("🌐 Web server started on port 80");
}

// ==================== WEB HANDLERS ====================
void handleRoot() {
  Serial.println("🌐 Root page requested");
  
  // Initialize SPIFFS if not already done
  if (!SPIFFS.begin()) {
    Serial.println("❌ SPIFFS Mount Failed - trying to format...");
    if (SPIFFS.format()) {
      Serial.println("✅ SPIFFS formatted successfully");
      if (SPIFFS.begin()) {
        Serial.println("✅ SPIFFS mounted after format");
      } else {
        Serial.println("❌ SPIFFS mount failed even after format");
      }
    } else {
      Serial.println("❌ SPIFFS format failed");
    }
  } else {
    Serial.println("✅ SPIFFS mounted successfully");
  }
  
  // List files in SPIFFS for debugging
  Serial.println("📁 Files in SPIFFS:");
  Dir dir = SPIFFS.openDir("/");
  bool hasFiles = false;
  while (dir.next()) {
    hasFiles = true;
    Serial.print("  - ");
    Serial.print(dir.fileName());
    Serial.print(" (");
    Serial.print(dir.fileSize());
    Serial.println(" bytes)");
  }
  
  if (!hasFiles) {
    Serial.println("  📁 No files found in SPIFFS");
  }
  
  // Try to serve the external HTML file
  if (SPIFFS.exists("/arduino.html")) {
    Serial.println("✅ arduino.html found in SPIFFS");
    File file = SPIFFS.open("/arduino.html", "r");
    if (file) {
      Serial.println("✅ File opened successfully");
      String contentType = "text/html";
      if (server.streamFile(file, contentType) != file.size()) {
        Serial.println("❌ Error streaming file");
      } else {
        Serial.println("🌐 External HTML file served successfully");
      }
      file.close();
      return;
    } else {
      Serial.println("❌ Failed to open arduino.html");
    }
  } else {
    Serial.println("❌ arduino.html not found in SPIFFS");
    Serial.println("💡 To use the full interface:");
    Serial.println("   1. Create a 'data' folder in your sketch directory");
    Serial.println("   2. Copy arduino.html to the data folder");
    Serial.println("   3. Use Tools -> ESP8266 Sketch Data Upload");
    Serial.println("   4. Reset the ESP8266 and refresh the page");
  }
  
  // Fallback to minimal HTML if file not found
  String html = R"(
<!DOCTYPE html>
<html>
<head>
    <title>ESP8266 Server</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; background: #f0f0f0; }
        .container { max-width: 800px; margin: 0 auto; background: white; padding: 20px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        h1 { color: #333; text-align: center; margin-bottom: 30px; }
        .alert { padding: 15px; margin: 20px 0; border-radius: 5px; background: #fff3cd; border: 1px solid #ffeaa7; color: #856404; }
        .status { margin: 10px 0; padding: 15px; background: #e9ecef; border-radius: 8px; border-left: 4px solid #007bff; }
        .controls { margin: 20px 0; text-align: center; }
        .btn { padding: 12px 24px; margin: 8px; background: #007bff; color: white; border: none; border-radius: 5px; cursor: pointer; text-decoration: none; display: inline-block; font-size: 14px; }
        .btn:hover { background: #0056b3; }
        .btn-success { background: #28a745; }
        .btn-success:hover { background: #1e7e34; }
        .btn-warning { background: #ffc107; color: #212529; }
        .btn-warning:hover { background: #e0a800; }
        .instructions { background: #f8f9fa; padding: 20px; border-radius: 8px; margin: 20px 0; }
        .instructions h3 { color: #495057; margin-bottom: 15px; }
        .instructions ol { margin-left: 20px; }
        .instructions li { margin: 8px 0; line-height: 1.6; }
        .code { background: #f1f3f4; padding: 2px 6px; border-radius: 3px; font-family: monospace; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🌐 ESP8266 Complete Server</h1>
        
        <div class="alert">
            <strong>⚠️ Notice:</strong> External HTML file (arduino.html) not found in SPIFFS. Using fallback interface.
            <br><br>
            <strong>🔧 Current Status:</strong> The ESP8266 is looking for <code>/arduino.html</code> in its flash memory (SPIFFS).
        </div>
        
        <div class="instructions">
            <h3>📋 How to upload the full HTML interface:</h3>
            <ol>
                <li><strong>Prepare the data folder:</strong> Create a folder called <span class="code">data</span> in your Arduino sketch folder (same folder as your .ino file)</li>
                <li><strong>Copy the HTML file:</strong> Copy <span class="code">arduino.html</span> from the <span class="code">static</span> folder into the <span class="code">data</span> folder</li>
                <li><strong>Install the upload tool:</strong> In Arduino IDE, go to <strong>Tools</strong> → <strong>Manage Tools</strong> → Search for "ESP8266 Sketch Data Upload" and install it</li>
                <li><strong>Upload the data:</strong> Go to <strong>Tools</strong> → <strong>ESP8266 Sketch Data Upload</strong> and wait for completion</li>
                <li><strong>Reset and refresh:</strong> Reset the ESP8266 and refresh this page to see the full interface</li>
            </ol>
            <p><strong>💡 Alternative:</strong> You can also use the <strong>ESP8266 LittleFS Data Upload</strong> tool if available.</p>
        </div>
        
        <div class="status">
            <strong>🔧 Quick Controls (API Testing):</strong>
        </div>
        
        <div class="controls">
            <a href="/api/status" class="btn btn-success">📊 Status API</a>
            <a href="/api/system" class="btn btn-warning">💻 System Info</a>
            <a href="/api/wifi" class="btn btn-success">📶 WiFi Info</a>
        </div>
        
        <div class="controls">
            <a href="/led/toggle" class="btn">💡 Toggle LED</a>
            <a href="/relay/toggle" class="btn">🔌 Toggle Relay</a>
        </div>
        
        <div class="status">
            <strong>📡 Device Info:</strong><br>
            IP: )";
  html += WiFi.localIP().toString();
  html += R"(<br>
            MAC: )";
  html += WiFi.macAddress();
  html += R"(<br>
            Chip ID: 0x)";
  html += String(ESP.getChipId(), HEX);
  html += R"(<br>
            Free Heap: )";
  html += formatBytes(ESP.getFreeHeap());
  html += R"(
        </div>
    </div>
</body>
</html>
  )";
  
  server.send(200, "text/html", html);
  Serial.println("🌐 Fallback HTML served (external file not found)");
}

void handleApiStatus() {
  String response = createStatusJson();
  server.send(200, "application/json", response);
  Serial.println("📡 API: Status requested");
}

void handleApiControl() {
  String response = createControlJson();
  server.send(200, "application/json", response);
  Serial.println("📡 API: Control status requested");
}

void handleApiControlPost() {
  String message = "";
  bool success = false;
  
  if (server.hasArg("led")) {
    String ledCmd = server.arg("led");
    if (ledCmd == "on") { setLED(true); success = true; message = "LED turned ON"; }
    else if (ledCmd == "off") { setLED(false); success = true; message = "LED turned OFF"; }
    else if (ledCmd == "toggle") { toggleLED(); success = true; message = "LED toggled"; }
  }
  
  if (server.hasArg("relay")) {
    String relayCmd = server.arg("relay");
    if (relayCmd == "on") { setRelay(true); success = true; message = "Relay turned ON"; }
    else if (relayCmd == "off") { setRelay(false); success = true; message = "Relay turned OFF"; }
    else if (relayCmd == "toggle") { toggleRelay(); success = true; message = "Relay toggled"; }
  }
  
  if (server.hasArg("pwm")) {
    int pwmVal = server.arg("pwm").toInt();
    setPWM(pwmVal);
    success = true;
    message = "PWM set to " + String(pwmVal);
  }
  
  String response = createJsonResponse(success ? "ok" : "error", message);
  server.send(success ? 200 : 400, "application/json", response);
  Serial.println("📡 API: Control command - " + message);
}

void handleApiSystem() {
  String json = "{";
  json += "\"chip_id\":" + String(ESP.getChipId()) + ",";
  json += "\"flash_size\":" + String(ESP.getFlashChipSize()) + ",";
  json += "\"free_heap\":" + String(ESP.getFreeHeap()) + ",";
  json += "\"cpu_freq\":" + String(ESP.getCpuFreqMHz()) + ",";
  json += "\"sketch_size\":" + String(ESP.getSketchSize()) + ",";
  json += "\"boot_mode\":" + String(ESP.getBootMode()) + ",";
  json += "\"reset_reason\":\"" + ESP.getResetReason() + "\"";
  json += "}";
  
  server.send(200, "application/json", json);
  Serial.println("📡 API: System info requested");
}

void handleApiWifi() {
  String json = "{";
  json += "\"ssid\":\"" + WiFi.SSID() + "\",";
  json += "\"ip\":\"" + WiFi.localIP().toString() + "\",";
  json += "\"mac\":\"" + WiFi.macAddress() + "\",";
  json += "\"rssi\":" + String(WiFi.RSSI()) + ",";
  json += "\"channel\":" + String(WiFi.channel()) + ",";
  json += "\"reconnect_count\":" + String(wifiReconnectCount);
  json += "}";
  
  server.send(200, "application/json", json);
  Serial.println("📡 API: WiFi info requested");
}

void handleApiReset() {
  String response = createJsonResponse("ok", "System will reset in 2 seconds");
  server.send(200, "application/json", response);
  
  Serial.println("🔄 System reset requested");
  delay(2000);
  ESP.restart();
}

void handle404() {
  String response = createJsonResponse("error", "Endpoint not found");
  server.send(404, "application/json", response);
  Serial.println("❌ 404 - " + server.uri());
}

// ==================== SERIAL FILE UPLOAD HANDLER ====================
void handleSerialUpload() {
  if (Serial.available()) {
    String command = Serial.readStringUntil('\n');
    command.trim();
    
    if (command.startsWith("UPLOAD_FILE:")) {
      String filePath = command.substring(12);
      Serial.print("READY_FOR_FILE:");
      Serial.println(filePath);
      
      // Wait for file size
      unsigned long sizeTimeout = millis() + 5000;
      while (!Serial.available() && millis() < sizeTimeout) {
        delay(10);
      }
      
      if (Serial.available()) {
        String sizeCommand = Serial.readStringUntil('\n');
        sizeCommand.trim();
        
        if (sizeCommand.startsWith("SIZE:")) {
          int fileSize = sizeCommand.substring(5).toInt();
          Serial.print("READY_FOR_DATA:");
          Serial.println(fileSize);
          
          // Prepare to receive file data
          String fileData = "";
          fileData.reserve(fileSize + 100);
          
          unsigned long startTime = millis();
          while (fileData.length() < fileSize && (millis() - startTime) < 30000) {
            if (Serial.available()) {
              char c = Serial.read();
              fileData += c;
            }
          }
          
          // Check for end marker
          String endMarker = "";
          unsigned long endTimeout = millis() + 2000;
          while (millis() < endTimeout && endMarker.indexOf("END_UPLOAD") < 0) {
            if (Serial.available()) {
              endMarker += (char)Serial.read();
            }
          }
          
          if (endMarker.indexOf("END_UPLOAD") >= 0) {
            // Initialize SPIFFS if needed
            if (!SPIFFS.begin()) {
              SPIFFS.format();
              SPIFFS.begin();
            }
            
            // Save file to SPIFFS
            File file = SPIFFS.open(filePath, "w");
            if (file) {
              file.print(fileData);
              file.close();
              Serial.print("UPLOAD_SUCCESS:");
              Serial.println(filePath);
              Serial.print("FILE_SIZE:");
              Serial.println(fileData.length());
              
              // If it's a config file, reload configuration
              if (filePath == "/wifi_config.json") {
                Serial.println("🔄 Reloading configuration...");
                loadConfig();
              }
            } else {
              Serial.print("UPLOAD_ERROR:");
              Serial.println(filePath);
            }
          } else {
            Serial.println("UPLOAD_ERROR: End marker not received");
          }
        }
      } else {
        Serial.println("UPLOAD_ERROR: Size command timeout");
      }
    }
  }
}
