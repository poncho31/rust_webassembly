// ESP8266 Complete Server - No External Libraries
// Complete WiFi + Web Server + API + Sensors + Controls for NodeMCU/ESP8266
// Uses only built-in ESP8266 libraries - no external dependencies!

#include <ESP8266WiFi.h>
#include <ESP8266WebServer.h>
#include <ESP8266mDNS.h>
#include <EEPROM.h>
#include <WiFiUdp.h>
#include <Ticker.h>

// ==================== CONFIGURATION ====================
// WiFi credentials - CHANGE THESE TO YOUR NETWORK
const char* ssid = "TADAAM-BE93TA3";
const char* password = "97EG82R3D357";

// Device configuration
const char* deviceName = "ESP8266-Complete";
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
    Serial.println("══════════════════════════════════════════════");
    Serial.print("🌐 WEB SERVER READY: http://");
    Serial.println(WiFi.localIP());
    Serial.println("══════════════════════════════════════════════");
    Serial.print("📡 IP address: ");
    Serial.println(WiFi.localIP());
    Serial.print("📶 Signal strength: ");
    Serial.print(WiFi.RSSI());
    Serial.println(" dBm");
    Serial.println("══════════════════════════════════════════════");
    Serial.println("🔗 Access the web interface at:");
    Serial.print("   http://");
    Serial.println(WiFi.localIP());
    Serial.println("══════════════════════════════════════════════");
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
  String html = R"(
<!DOCTYPE html>
<html>
<head>
    <title>ESP8266 Complete Server</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: Arial, sans-serif; background: #f0f0f0; }
        .container { max-width: 1000px; margin: 0 auto; padding: 20px; }
        .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; border-radius: 10px; margin-bottom: 20px; text-align: center; }
        .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px; }
        .card { background: white; padding: 20px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        .card h3 { color: #333; margin-bottom: 15px; }
        .status-item { padding: 10px; background: #f8f9fa; border-radius: 5px; margin: 5px 0; }
        .controls { display: flex; gap: 10px; flex-wrap: wrap; margin-top: 15px; }
        .btn { padding: 10px 15px; border: none; border-radius: 5px; cursor: pointer; text-decoration: none; display: inline-block; transition: all 0.3s; }
        .btn-primary { background: #007bff; color: white; }
        .btn-success { background: #28a745; color: white; }
        .btn-warning { background: #ffc107; color: #333; }
        .btn-danger { background: #dc3545; color: white; }
        .btn:hover { opacity: 0.8; transform: translateY(-1px); }
        .sensor-value { font-size: 1.8em; font-weight: bold; color: #007bff; }
        @media (max-width: 768px) { .grid { grid-template-columns: 1fr; } }
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
        setInterval(updateStatus, 3000);
        window.onload = updateStatus;
    </script>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🌐 ESP8266 Complete Server</h1>
            <p>No External Dependencies • Version 1.0.0</p>
        </div>
        
        <div class="grid">
            <div class="card">
                <h3>📊 System Status</h3>
                <div class="status-item"><strong>Uptime:</strong> <span id="uptime">)";
  html += String(uptime) + R"(s</span></div>
                <div class="status-item"><strong>Free Heap:</strong> <span id="heap">)";
  html += String(ESP.getFreeHeap()) + R"( bytes</span></div>
                <div class="status-item"><strong>WiFi RSSI:</strong> <span id="wifi-rssi">)";
  html += String(WiFi.RSSI()) + R"( dBm</span></div>
                <div class="status-item"><strong>IP:</strong> )";
  html += WiFi.localIP().toString() + R"(</div>
            </div>
            
            <div class="card">
                <h3>🔧 Device Control</h3>
                <div class="status-item"><strong>LED:</strong> <span id="led-status">)";
  html += (ledState ? "ON" : "OFF") + String(R"(</span></div>
                <div class="status-item"><strong>Relay:</strong> <span id="relay-status">)");
  html += (relayState ? "ON" : "OFF") + String(R"(</span></div>
                <div class="controls">
                    <a href="/led/toggle" class="btn btn-primary">💡 Toggle LED</a>
                    <a href="/relay/toggle" class="btn btn-warning">🔌 Toggle Relay</a>
                </div>
            </div>
            
            <div class="card">
                <h3>📡 Sensors</h3>
                <div class="status-item"><strong>Analog:</strong> <span class="sensor-value" id="analog">)");
  html += String(analogValue) + R"(</span></div>
                <div class="status-item"><strong>Digital:</strong> )";
  html += String(sensorValue) + R"(</div>
                <div class="status-item"><strong>PWM:</strong> )";
  html += String(pwmValue) + R"(</div>
            </div>
            
            <div class="card">
                <h3>🛠️ API Endpoints</h3>
                <div class="controls">
                    <a href="/api/status" class="btn btn-success">📊 Status</a>
                    <a href="/api/control" class="btn btn-primary">🔧 Control</a>
                    <a href="/api/system" class="btn btn-warning">💻 System</a>
                    <a href="/api/wifi" class="btn btn-success">📡 WiFi</a>
                </div>
            </div>
        </div>
    </div>
</body>
</html>
  )";
  
  server.send(200, "text/html", html);
  Serial.println("🌐 Main dashboard served");
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
