// ESP8266 All-in-One Complete Server
// Complete WiFi + Web Server + API + Sensors + Controls for NodeMCU/ESP8266
// One sketch to rule them all!

#include <ESP8266WiFi.h>
#include <ESP8266WebServer.h>
#include <ESP8266mDNS.h>
#include <ArduinoJson.h>
#include <EEPROM.h>
#include <ESP8266HTTPClient.h>
#include <WiFiUdp.h>
#include <NTPClient.h>
#include <Ticker.h>

// ==================== CONFIGURATION ====================
// WiFi credentials - CHANGE THESE TO YOUR NETWORK
const char* ssid = "TADAAM-BE93TA3";
const char* password = "97EG82R3D357";

// Device configuration
const char* deviceName = "ESP8266-AllInOne";
const char* deviceVersion = "1.0.0";

// Pin definitions
const int ledPin = LED_BUILTIN;
const int analogPin = A0;
const int relayPin = 12;     // D6 on NodeMCU
const int buttonPin = 0;     // D3 on NodeMCU (Flash button)
const int sensorPin = 14;    // D5 on NodeMCU
const int pwmPin = 13;       // D7 on NodeMCU

// ==================== GLOBAL VARIABLES ====================
ESP8266WebServer server(80);
WiFiUDP ntpUDP;
NTPClient timeClient(ntpUDP, "pool.ntp.org", 0, 60000);
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
float cpuUsage = 0;
int freeHeap = 0;
String lastError = "";

// Settings (stored in EEPROM)
struct Settings {
  char deviceName[32];
  int sensorInterval;
  bool autoRelay;
  int pwmDefault;
  bool enableNTP;
  int timezone;
} settings;

// ==================== SETUP ====================
void setup() {
  Serial.begin(115200);
  Serial.println();
  Serial.println("==============================================");
  Serial.println("🚀 ESP8266 All-in-One Server Starting...");
  Serial.println("==============================================");
  
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
  
  // Handle NTP
  if (settings.enableNTP) {
    timeClient.update();
  }
  
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
  WiFi.begin(ssid, password);
  
  Serial.print("🔄 Connecting to WiFi");
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
    Serial.print("📡 IP address: ");
    Serial.println(WiFi.localIP());
    Serial.print("📶 Signal strength: ");
    Serial.print(WiFi.RSSI());
    Serial.println(" dBm");
  } else {
    Serial.println();
    Serial.println("❌ WiFi connection failed!");
    lastError = "WiFi connection failed";
  }
}

void initializeServices() {
  // Start mDNS
  if (MDNS.begin(deviceName)) {
    Serial.println("🌐 mDNS responder started");
    Serial.print("🔗 Access via: http://");
    Serial.print(deviceName);
    Serial.println(".local");
    
    // Add service discovery
    MDNS.addService("http", "tcp", 80);
    MDNS.addService("arduino", "tcp", 80);
  }
  
  // Start NTP client
  if (settings.enableNTP) {
    timeClient.begin();
    timeClient.setTimeOffset(settings.timezone * 3600);
    Serial.println("⏰ NTP client started");
  }
}

// ==================== SETTINGS MANAGEMENT ====================
void loadSettings() {
  EEPROM.get(0, settings);
  
  // Check if settings are valid (simple magic number check)
  if (strlen(settings.deviceName) == 0 || strlen(settings.deviceName) > 31) {
    // Load default settings
    strcpy(settings.deviceName, deviceName);
    settings.sensorInterval = 5000;
    settings.autoRelay = false;
    settings.pwmDefault = 128;
    settings.enableNTP = true;
    settings.timezone = 0;
    
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
      WiFi.begin(ssid, password);
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
  freeHeap = ESP.getFreeHeap();
  
  // Simple CPU usage calculation (approximation)
  static unsigned long lastTime = 0;
  unsigned long currentTime = micros();
  if (lastTime > 0) {
    cpuUsage = (currentTime - lastTime) / 10000.0; // Rough approximation
  }
  lastTime = currentTime;
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
  server.on("/api/settings", HTTP_GET, handleApiSettings);
  server.on("/api/settings", HTTP_POST, handleApiSettingsPost);
  server.on("/api/system", HTTP_GET, handleApiSystem);
  server.on("/api/wifi", HTTP_GET, handleApiWifi);
  server.on("/api/reset", HTTP_POST, handleApiReset);
  
  // Control endpoints (for easy access)
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
  String html = R"(
<!DOCTYPE html>
<html>
<head>
    <title>)" + String(settings.deviceName) + R"( - ESP8266 All-in-One</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; background: #f5f5f5; }
        .container { max-width: 1200px; margin: 0 auto; padding: 20px; }
        .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; border-radius: 15px; margin-bottom: 30px; text-align: center; }
        .header h1 { font-size: 2.5em; margin-bottom: 10px; }
        .header p { opacity: 0.9; }
        .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; margin-bottom: 30px; }
        .card { background: white; padding: 25px; border-radius: 15px; box-shadow: 0 5px 15px rgba(0,0,0,0.1); }
        .card h3 { color: #333; margin-bottom: 20px; font-size: 1.3em; }
        .status-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin-bottom: 20px; }
        .status-item { padding: 10px; background: #f8f9fa; border-radius: 8px; }
        .status-item strong { color: #495057; }
        .controls { display: flex; gap: 10px; flex-wrap: wrap; }
        .btn { padding: 12px 24px; border: none; border-radius: 8px; cursor: pointer; font-size: 14px; transition: all 0.3s; text-decoration: none; display: inline-block; text-align: center; }
        .btn-primary { background: #007bff; color: white; }
        .btn-success { background: #28a745; color: white; }
        .btn-danger { background: #dc3545; color: white; }
        .btn-warning { background: #ffc107; color: #212529; }
        .btn-info { background: #17a2b8; color: white; }
        .btn:hover { transform: translateY(-2px); box-shadow: 0 4px 8px rgba(0,0,0,0.2); }
        .sensor-value { font-size: 2em; font-weight: bold; color: #007bff; }
        .footer { text-align: center; color: #666; margin-top: 40px; }
        .online { color: #28a745; }
        .offline { color: #dc3545; }
        @media (max-width: 768px) {
            .grid { grid-template-columns: 1fr; }
            .controls { justify-content: center; }
        }
    </style>
    <script>
        function updateStatus() {
            fetch('/api/status')
                .then(response => response.json())
                .then(data => {
                    document.getElementById('uptime').textContent = data.uptime + 's';
                    document.getElementById('heap').textContent = data.free_heap + ' bytes';
                    document.getElementById('analog').textContent = data.analog_value;
                    document.getElementById('led-status').textContent = data.led_state ? 'ON' : 'OFF';
                    document.getElementById('relay-status').textContent = data.relay_state ? 'ON' : 'OFF';
                    document.getElementById('wifi-rssi').textContent = data.wifi_rssi + ' dBm';
                })
                .catch(error => console.error('Error:', error));
        }
        
        setInterval(updateStatus, 2000);
        window.onload = updateStatus;
    </script>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🌐 )" + String(settings.deviceName) + R"(</h1>
            <p>ESP8266 All-in-One Server • Version )" + String(deviceVersion) + R"(</p>
        </div>
        
        <div class="grid">
            <div class="card">
                <h3>📊 System Status</h3>
                <div class="status-grid">
                    <div class="status-item">
                        <strong>Uptime:</strong><br>
                        <span id="uptime">)" + String(uptime) + R"(s</span>
                    </div>
                    <div class="status-item">
                        <strong>Free Heap:</strong><br>
                        <span id="heap">)" + String(ESP.getFreeHeap()) + R"( bytes</span>
                    </div>
                    <div class="status-item">
                        <strong>WiFi RSSI:</strong><br>
                        <span id="wifi-rssi">)" + String(WiFi.RSSI()) + R"( dBm</span>
                    </div>
                    <div class="status-item">
                        <strong>IP Address:</strong><br>
                        )" + WiFi.localIP().toString() + R"(
                    </div>
                </div>
            </div>
            
            <div class="card">
                <h3>🔧 Device Control</h3>
                <div class="status-grid">
                    <div class="status-item">
                        <strong>LED:</strong><br>
                        <span id="led-status">)" + (ledState ? "ON" : "OFF") + R"(</span>
                    </div>
                    <div class="status-item">
                        <strong>Relay:</strong><br>
                        <span id="relay-status">)" + (relayState ? "ON" : "OFF") + R"(</span>
                    </div>
                </div>
                <div class="controls">
                    <a href="/led/toggle" class="btn btn-primary">💡 Toggle LED</a>
                    <a href="/relay/toggle" class="btn btn-warning">🔌 Toggle Relay</a>
                </div>
            </div>
            
            <div class="card">
                <h3>📡 Sensors</h3>
                <div class="status-item">
                    <strong>Analog Value:</strong><br>
                    <span class="sensor-value" id="analog">)" + String(analogValue) + R"(</span>
                </div>
                <div class="controls">
                    <a href="/api/control" class="btn btn-info">📈 Get All Values</a>
                </div>
            </div>
            
            <div class="card">
                <h3>🛠️ API Endpoints</h3>
                <div class="controls">
                    <a href="/api/status" class="btn btn-info">📊 Status</a>
                    <a href="/api/control" class="btn btn-success">🔧 Control</a>
                    <a href="/api/settings" class="btn btn-warning">⚙️ Settings</a>
                    <a href="/api/system" class="btn btn-primary">💻 System</a>
                </div>
            </div>
        </div>
        
        <div class="footer">
            <p>ESP8266 All-in-One Server • Built with ❤️ • Auto-refresh every 2s</p>
        </div>
    </div>
</body>
</html>
  )";
  
  server.send(200, "text/html", html);
  Serial.println("🌐 Main dashboard served");
}

void handleApiStatus() {
  DynamicJsonDocument doc(1024);
  
  doc["status"] = "ok";
  doc["device_name"] = settings.deviceName;
  doc["version"] = deviceVersion;
  doc["uptime"] = uptime;
  doc["free_heap"] = ESP.getFreeHeap();
  doc["chip_id"] = ESP.getChipId();
  doc["flash_size"] = ESP.getFlashChipSize();
  doc["led_state"] = ledState;
  doc["relay_state"] = relayState;
  doc["button_state"] = buttonState;
  doc["analog_value"] = analogValue;
  doc["sensor_value"] = sensorValue;
  doc["pwm_value"] = pwmValue;
  doc["wifi_rssi"] = WiFi.RSSI();
  doc["wifi_reconnects"] = wifiReconnectCount;
  doc["last_error"] = lastError;
  
  if (settings.enableNTP) {
    doc["current_time"] = timeClient.getFormattedTime();
    doc["epoch_time"] = timeClient.getEpochTime();
  }
  
  String response;
  serializeJson(doc, response);
  server.send(200, "application/json", response);
  Serial.println("📡 API: Status requested");
}

void handleApiControl() {
  DynamicJsonDocument doc(512);
  
  doc["led_state"] = ledState;
  doc["relay_state"] = relayState;
  doc["button_state"] = buttonState;
  doc["analog_value"] = analogValue;
  doc["sensor_value"] = sensorValue;
  doc["pwm_value"] = pwmValue;
  
  String response;
  serializeJson(doc, response);
  server.send(200, "application/json", response);
  Serial.println("📡 API: Control status requested");
}

void handleApiControlPost() {
  DynamicJsonDocument doc(256);
  bool success = false;
  String message = "";
  
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
  
  doc["status"] = success ? "ok" : "error";
  doc["message"] = message;
  doc["led_state"] = ledState;
  doc["relay_state"] = relayState;
  doc["pwm_value"] = pwmValue;
  
  String response;
  serializeJson(doc, response);
  server.send(success ? 200 : 400, "application/json", response);
  Serial.println("📡 API: Control command - " + message);
}

void handleApiSettings() {
  DynamicJsonDocument doc(512);
  
  doc["device_name"] = settings.deviceName;
  doc["sensor_interval"] = settings.sensorInterval;
  doc["auto_relay"] = settings.autoRelay;
  doc["pwm_default"] = settings.pwmDefault;
  doc["enable_ntp"] = settings.enableNTP;
  doc["timezone"] = settings.timezone;
  
  String response;
  serializeJson(doc, response);
  server.send(200, "application/json", response);
  Serial.println("📡 API: Settings requested");
}

void handleApiSettingsPost() {
  bool changed = false;
  
  if (server.hasArg("device_name")) {
    String newName = server.arg("device_name");
    if (newName.length() > 0 && newName.length() < 32) {
      strcpy(settings.deviceName, newName.c_str());
      changed = true;
    }
  }
  
  if (server.hasArg("sensor_interval")) {
    settings.sensorInterval = server.arg("sensor_interval").toInt();
    changed = true;
  }
  
  if (server.hasArg("auto_relay")) {
    settings.autoRelay = server.arg("auto_relay") == "true";
    changed = true;
  }
  
  if (server.hasArg("pwm_default")) {
    settings.pwmDefault = server.arg("pwm_default").toInt();
    changed = true;
  }
  
  if (server.hasArg("enable_ntp")) {
    settings.enableNTP = server.arg("enable_ntp") == "true";
    changed = true;
  }
  
  if (server.hasArg("timezone")) {
    settings.timezone = server.arg("timezone").toInt();
    changed = true;
  }
  
  if (changed) {
    saveSettings();
  }
  
  DynamicJsonDocument doc(256);
  doc["status"] = "ok";
  doc["message"] = changed ? "Settings saved" : "No changes";
  doc["changed"] = changed;
  
  String response;
  serializeJson(doc, response);
  server.send(200, "application/json", response);
  Serial.println("📡 API: Settings " + String(changed ? "saved" : "unchanged"));
}

void handleApiSystem() {
  DynamicJsonDocument doc(1024);
  
  doc["chip_id"] = ESP.getChipId();
  doc["flash_id"] = ESP.getFlashChipId();
  doc["flash_size"] = ESP.getFlashChipSize();
  doc["flash_speed"] = ESP.getFlashChipSpeed();
  doc["free_heap"] = ESP.getFreeHeap();
  doc["heap_fragmentation"] = ESP.getHeapFragmentation();
  doc["max_free_block"] = ESP.getMaxFreeBlockSize();
  doc["cpu_freq"] = ESP.getCpuFreqMHz();
  doc["sketch_size"] = ESP.getSketchSize();
  doc["free_sketch_space"] = ESP.getFreeSketchSpace();
  doc["reset_reason"] = ESP.getResetReason();
  doc["boot_version"] = ESP.getBootVersion();
  doc["boot_mode"] = ESP.getBootMode();
  
  String response;
  serializeJson(doc, response);
  server.send(200, "application/json", response);
  Serial.println("📡 API: System info requested");
}

void handleApiWifi() {
  DynamicJsonDocument doc(512);
  
  doc["ssid"] = WiFi.SSID();
  doc["ip"] = WiFi.localIP().toString();
  doc["gateway"] = WiFi.gatewayIP().toString();
  doc["subnet"] = WiFi.subnetMask().toString();
  doc["dns"] = WiFi.dnsIP().toString();
  doc["mac"] = WiFi.macAddress();
  doc["rssi"] = WiFi.RSSI();
  doc["channel"] = WiFi.channel();
  doc["hostname"] = WiFi.hostname();
  doc["reconnect_count"] = wifiReconnectCount;
  doc["status"] = WiFi.status();
  
  String response;
  serializeJson(doc, response);
  server.send(200, "application/json", response);
  Serial.println("📡 API: WiFi info requested");
}

void handleApiReset() {
  DynamicJsonDocument doc(256);
  doc["status"] = "ok";
  doc["message"] = "System will reset in 2 seconds";
  
  String response;
  serializeJson(doc, response);
  server.send(200, "application/json", response);
  
  Serial.println("🔄 System reset requested");
  delay(2000);
  ESP.restart();
}

void handle404() {
  DynamicJsonDocument doc(256);
  doc["status"] = "error";
  doc["message"] = "Endpoint not found";
  doc["uri"] = server.uri();
  doc["method"] = (server.method() == HTTP_GET) ? "GET" : "POST";
  
  String response;
  serializeJson(doc, response);
  server.send(404, "application/json", response);
  Serial.println("❌ 404 - " + server.uri());
}
