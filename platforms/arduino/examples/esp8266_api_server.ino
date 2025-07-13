// ESP8266 Simple API Server
// Lightweight REST API server for NodeMCU/ESP8266

#include <ESP8266WiFi.h>
#include <ESP8266WebServer.h>
#include <ArduinoJson.h>

// WiFi credentials - CHANGE THESE TO YOUR NETWORK
const char* ssid = "YOUR_WIFI_SSID";
const char* password = "YOUR_WIFI_PASSWORD";

// Web server on port 80
ESP8266WebServer server(80);

// Pins and states
const int ledPin = LED_BUILTIN;
const int analogPin = A0;
bool ledState = false;

void setup() {
  Serial.begin(115200);
  Serial.println();
  Serial.println("ESP8266 API Server Starting...");
  
  // Initialize pins
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
  
  // Setup API routes
  setupRoutes();
  
  // Start server
  server.begin();
  Serial.println("API Server started");
  Serial.println("====================================");
  Serial.println("API Endpoints:");
  Serial.println("  GET  /api/status    - Get system status");
  Serial.println("  GET  /api/led       - Get LED state");
  Serial.println("  POST /api/led       - Set LED state");
  Serial.println("  GET  /api/analog    - Get analog reading");
  Serial.println("  GET  /api/wifi      - Get WiFi info");
  Serial.println("====================================");
}

void loop() {
  server.handleClient();
  delay(10);
}

void setupRoutes() {
  // Enable CORS for all responses
  server.enableCORS(true);
  
  // API Status
  server.on("/api/status", HTTP_GET, []() {
    DynamicJsonDocument doc(1024);
    doc["status"] = "ok";
    doc["uptime"] = millis() / 1000;
    doc["free_heap"] = ESP.getFreeHeap();
    doc["chip_id"] = ESP.getChipId();
    doc["flash_size"] = ESP.getFlashChipSize();
    doc["led_state"] = ledState;
    doc["analog_reading"] = analogRead(analogPin);
    
    String response;
    serializeJson(doc, response);
    server.send(200, "application/json", response);
    Serial.println("API: Status requested");
  });
  
  // LED Control - GET
  server.on("/api/led", HTTP_GET, []() {
    DynamicJsonDocument doc(256);
    doc["led_state"] = ledState;
    doc["pin"] = ledPin;
    
    String response;
    serializeJson(doc, response);
    server.send(200, "application/json", response);
    Serial.println("API: LED state requested");
  });
  
  // LED Control - POST
  server.on("/api/led", HTTP_POST, []() {
    if (server.hasArg("state")) {
      String state = server.arg("state");
      
      if (state == "on" || state == "true" || state == "1") {
        digitalWrite(ledPin, LOW); // Inverted logic
        ledState = true;
      } else if (state == "off" || state == "false" || state == "0") {
        digitalWrite(ledPin, HIGH); // Inverted logic
        ledState = false;
      } else if (state == "toggle") {
        ledState = !ledState;
        digitalWrite(ledPin, ledState ? LOW : HIGH);
      }
      
      DynamicJsonDocument doc(256);
      doc["status"] = "ok";
      doc["led_state"] = ledState;
      doc["message"] = "LED state changed";
      
      String response;
      serializeJson(doc, response);
      server.send(200, "application/json", response);
      Serial.println("API: LED state changed to " + String(ledState ? "ON" : "OFF"));
    } else {
      DynamicJsonDocument doc(256);
      doc["status"] = "error";
      doc["message"] = "Missing 'state' parameter";
      
      String response;
      serializeJson(doc, response);
      server.send(400, "application/json", response);
    }
  });
  
  // Analog Reading
  server.on("/api/analog", HTTP_GET, []() {
    int reading = analogRead(analogPin);
    float voltage = (reading * 3.3) / 1024.0;
    
    DynamicJsonDocument doc(256);
    doc["raw_value"] = reading;
    doc["voltage"] = voltage;
    doc["percentage"] = (reading * 100) / 1024;
    
    String response;
    serializeJson(doc, response);
    server.send(200, "application/json", response);
    Serial.println("API: Analog reading - " + String(reading) + " (" + String(voltage) + "V)");
  });
  
  // WiFi Info
  server.on("/api/wifi", HTTP_GET, []() {
    DynamicJsonDocument doc(512);
    doc["ssid"] = WiFi.SSID();
    doc["ip"] = WiFi.localIP().toString();
    doc["mac"] = WiFi.macAddress();
    doc["rssi"] = WiFi.RSSI();
    doc["channel"] = WiFi.channel();
    doc["hostname"] = WiFi.hostname();
    
    String response;
    serializeJson(doc, response);
    server.send(200, "application/json", response);
    Serial.println("API: WiFi info requested");
  });
  
  // Root endpoint with API documentation
  server.on("/", HTTP_GET, []() {
    String html = R"(
<!DOCTYPE html>
<html>
<head>
    <title>ESP8266 API Server</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        h1 { color: #333; }
        .endpoint { background: #f5f5f5; padding: 10px; margin: 10px 0; border-radius: 5px; }
        .method { font-weight: bold; color: #2196F3; }
        .url { font-family: monospace; background: #e8e8e8; padding: 2px 5px; border-radius: 3px; }
        button { padding: 5px 10px; margin: 2px; cursor: pointer; }
    </style>
</head>
<body>
    <h1>🌐 ESP8266 API Server</h1>
    <p>Simple REST API for ESP8266 control</p>
    
    <h2>Available Endpoints:</h2>
    
    <div class="endpoint">
        <span class="method">GET</span> <span class="url">/api/status</span><br>
        Get system status and health information
        <button onclick="fetch('/api/status').then(r=>r.json()).then(d=>alert(JSON.stringify(d,null,2)))">Test</button>
    </div>
    
    <div class="endpoint">
        <span class="method">GET</span> <span class="url">/api/led</span><br>
        Get current LED state
        <button onclick="fetch('/api/led').then(r=>r.json()).then(d=>alert(JSON.stringify(d,null,2)))">Test</button>
    </div>
    
    <div class="endpoint">
        <span class="method">POST</span> <span class="url">/api/led</span><br>
        Control LED state (parameters: state=on/off/toggle)
        <button onclick="fetch('/api/led', {method:'POST', headers:{'Content-Type':'application/x-www-form-urlencoded'}, body:'state=on'}).then(r=>r.json()).then(d=>alert(JSON.stringify(d,null,2)))">Turn ON</button>
        <button onclick="fetch('/api/led', {method:'POST', headers:{'Content-Type':'application/x-www-form-urlencoded'}, body:'state=off'}).then(r=>r.json()).then(d=>alert(JSON.stringify(d,null,2)))">Turn OFF</button>
        <button onclick="fetch('/api/led', {method:'POST', headers:{'Content-Type':'application/x-www-form-urlencoded'}, body:'state=toggle'}).then(r=>r.json()).then(d=>alert(JSON.stringify(d,null,2)))">Toggle</button>
    </div>
    
    <div class="endpoint">
        <span class="method">GET</span> <span class="url">/api/analog</span><br>
        Get analog pin reading (A0)
        <button onclick="fetch('/api/analog').then(r=>r.json()).then(d=>alert(JSON.stringify(d,null,2)))">Test</button>
    </div>
    
    <div class="endpoint">
        <span class="method">GET</span> <span class="url">/api/wifi</span><br>
        Get WiFi connection information
        <button onclick="fetch('/api/wifi').then(r=>r.json()).then(d=>alert(JSON.stringify(d,null,2)))">Test</button>
    </div>
    
    <h2>Usage Examples:</h2>
    <pre>
curl http://)" + WiFi.localIP().toString() + R"(/api/status
curl http://)" + WiFi.localIP().toString() + R"(/api/led
curl -X POST -d "state=on" http://)" + WiFi.localIP().toString() + R"(/api/led
curl http://)" + WiFi.localIP().toString() + R"(/api/analog
    </pre>
</body>
</html>
    )";
    server.send(200, "text/html", html);
  });
  
  // 404 handler
  server.onNotFound([]() {
    DynamicJsonDocument doc(256);
    doc["status"] = "error";
    doc["message"] = "Endpoint not found";
    doc["uri"] = server.uri();
    
    String response;
    serializeJson(doc, response);
    server.send(404, "application/json", response);
  });
}
