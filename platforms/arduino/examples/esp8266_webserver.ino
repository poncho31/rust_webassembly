// ESP8266 Web Server Example
// Simple web server with WiFi connection for NodeMCU/ESP8266
// Access the server at http://[IP_ADDRESS] after connection

#include <ESP8266WiFi.h>
#include <ESP8266WebServer.h>
#include <ESP8266mDNS.h>

// WiFi credentials - CHANGE THESE TO YOUR NETWORK
const char* ssid = "YOUR_WIFI_SSID";
const char* password = "YOUR_WIFI_PASSWORD";

// Web server on port 80
ESP8266WebServer server(80);

// LED pin (built-in LED on NodeMCU)
const int ledPin = LED_BUILTIN;
bool ledState = false;

void setup() {
  Serial.begin(115200);
  Serial.println();
  Serial.println("ESP8266 Web Server Starting...");
  
  // Initialize LED pin
  pinMode(ledPin, OUTPUT);
  digitalWrite(ledPin, HIGH); // Turn off LED (inverted logic)
  
  // Connect to WiFi
  WiFi.begin(ssid, password);
  Serial.print("Connecting to WiFi");
  
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  
  Serial.println();
  Serial.println("WiFi connected!");
  Serial.print("IP address: ");
  Serial.println(WiFi.localIP());
  
  // Start mDNS service
  if (MDNS.begin("esp8266")) {
    Serial.println("mDNS responder started");
    Serial.println("Access via: http://esp8266.local");
  }
  
  // Define web server routes
  server.on("/", handleRoot);
  server.on("/led/on", handleLedOn);
  server.on("/led/off", handleLedOff);
  server.on("/led/toggle", handleLedToggle);
  server.on("/status", handleStatus);
  server.onNotFound(handleNotFound);
  
  // Start server
  server.begin();
  Serial.println("HTTP server started");
  Serial.println("====================================");
  Serial.println("Available endpoints:");
  Serial.println("  / - Main page");
  Serial.println("  /led/on - Turn LED on");
  Serial.println("  /led/off - Turn LED off");
  Serial.println("  /led/toggle - Toggle LED");
  Serial.println("  /status - Get system status");
  Serial.println("====================================");
}

void loop() {
  server.handleClient();
  MDNS.update();
  delay(10);
}

// Main page with interactive controls
void handleRoot() {
  String html = R"(
<!DOCTYPE html>
<html>
<head>
    <title>ESP8266 Web Server</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; background-color: #f0f0f0; }
        .container { max-width: 600px; margin: 0 auto; background: white; padding: 20px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        h1 { color: #333; text-align: center; }
        .status { background: #e8f4f8; padding: 15px; border-radius: 5px; margin: 20px 0; }
        .controls { display: flex; gap: 10px; flex-wrap: wrap; justify-content: center; }
        button { padding: 10px 20px; font-size: 16px; border: none; border-radius: 5px; cursor: pointer; transition: background-color 0.3s; }
        .btn-on { background-color: #4CAF50; color: white; }
        .btn-off { background-color: #f44336; color: white; }
        .btn-toggle { background-color: #2196F3; color: white; }
        .btn-status { background-color: #FF9800; color: white; }
        button:hover { opacity: 0.8; }
        .info { margin: 20px 0; padding: 15px; background: #fff3cd; border-radius: 5px; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🌐 ESP8266 Web Server</h1>
        
        <div class="status">
            <h3>📊 System Status</h3>
            <p><strong>IP Address:</strong> )" + WiFi.localIP().toString() + R"(</p>
            <p><strong>WiFi SSID:</strong> )" + String(ssid) + R"(</p>
            <p><strong>Signal Strength:</strong> )" + String(WiFi.RSSI()) + R"( dBm</p>
            <p><strong>LED Status:</strong> )" + (ledState ? "ON" : "OFF") + R"(</p>
        </div>
        
        <div class="controls">
            <button class="btn-on" onclick="location.href='/led/on'">💡 Turn LED ON</button>
            <button class="btn-off" onclick="location.href='/led/off'">🔌 Turn LED OFF</button>
            <button class="btn-toggle" onclick="location.href='/led/toggle'">🔄 Toggle LED</button>
            <button class="btn-status" onclick="location.href='/status'">📈 Get Status</button>
        </div>
        
        <div class="info">
            <h3>ℹ️ API Endpoints</h3>
            <ul>
                <li><strong>GET /</strong> - This main page</li>
                <li><strong>GET /led/on</strong> - Turn LED on</li>
                <li><strong>GET /led/off</strong> - Turn LED off</li>
                <li><strong>GET /led/toggle</strong> - Toggle LED state</li>
                <li><strong>GET /status</strong> - Get JSON status</li>
            </ul>
        </div>
        
        <div class="info">
            <h3>🔧 Setup Instructions</h3>
            <ol>
                <li>Update WiFi credentials in the code</li>
                <li>Upload to ESP8266/NodeMCU</li>
                <li>Open Serial Monitor to see IP address</li>
                <li>Access via browser: http://[IP_ADDRESS] or http://esp8266.local</li>
            </ol>
        </div>
    </div>
    
    <script>
        // Auto-refresh status every 10 seconds
        setInterval(function() {
            if (window.location.pathname === '/') {
                location.reload();
            }
        }, 10000);
    </script>
</body>
</html>
  )";
  
  server.send(200, "text/html", html);
  Serial.println("Served main page");
}

// Turn LED on
void handleLedOn() {
  digitalWrite(ledPin, LOW); // Inverted logic for built-in LED
  ledState = true;
  server.send(200, "text/plain", "LED turned ON");
  Serial.println("LED turned ON via web");
}

// Turn LED off
void handleLedOff() {
  digitalWrite(ledPin, HIGH); // Inverted logic for built-in LED
  ledState = false;
  server.send(200, "text/plain", "LED turned OFF");
  Serial.println("LED turned OFF via web");
}

// Toggle LED
void handleLedToggle() {
  ledState = !ledState;
  digitalWrite(ledPin, ledState ? LOW : HIGH); // Inverted logic
  server.send(200, "text/plain", "LED toggled - now " + String(ledState ? "ON" : "OFF"));
  Serial.println("LED toggled via web - now " + String(ledState ? "ON" : "OFF"));
}

// Status endpoint (JSON)
void handleStatus() {
  String json = "{";
  json += "\"ip\":\"" + WiFi.localIP().toString() + "\",";
  json += "\"ssid\":\"" + String(ssid) + "\",";
  json += "\"rssi\":" + String(WiFi.RSSI()) + ",";
  json += "\"led_state\":" + String(ledState ? "true" : "false") + ",";
  json += "\"uptime\":" + String(millis()) + ",";
  json += "\"free_heap\":" + String(ESP.getFreeHeap()) + ",";
  json += "\"chip_id\":\"" + String(ESP.getChipId(), HEX) + "\"";
  json += "}";
  
  server.send(200, "application/json", json);
  Serial.println("Status requested via web");
}

// 404 handler
void handleNotFound() {
  String message = "File Not Found\n\n";
  message += "URI: " + server.uri() + "\n";
  message += "Method: " + String((server.method() == HTTP_GET) ? "GET" : "POST") + "\n";
  message += "Arguments: " + String(server.args()) + "\n";
  
  for (uint8_t i = 0; i < server.args(); i++) {
    message += " " + server.argName(i) + ": " + server.arg(i) + "\n";
  }
  
  server.send(404, "text/plain", message);
  Serial.println("404 - Page not found: " + server.uri());
}
