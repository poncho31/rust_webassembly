// ESP8266 Simple WiFi Connection Test
// Basic WiFi connection and LED control for NodeMCU/ESP8266

#include <ESP8266WiFi.h>

// WiFi credentials - CHANGE THESE TO YOUR NETWORK
const char* ssid = "YOUR_WIFI_SSID";
const char* password = "YOUR_WIFI_PASSWORD";

// LED pin (built-in LED on NodeMCU)
const int ledPin = LED_BUILTIN;

void setup() {
  Serial.begin(115200);
  Serial.println();
  Serial.println("ESP8266 WiFi Test Starting...");
  
  // Initialize LED pin
  pinMode(ledPin, OUTPUT);
  digitalWrite(ledPin, HIGH); // Turn off LED (inverted logic)
  
  // Connect to WiFi
  WiFi.begin(ssid, password);
  Serial.print("Connecting to WiFi");
  
  // Blink LED while connecting
  while (WiFi.status() != WL_CONNECTED) {
    digitalWrite(ledPin, LOW);  // Turn on LED
    delay(250);
    digitalWrite(ledPin, HIGH); // Turn off LED
    delay(250);
    Serial.print(".");
  }
  
  Serial.println();
  Serial.println("WiFi connected successfully!");
  Serial.print("IP address: ");
  Serial.println(WiFi.localIP());
  Serial.print("MAC address: ");
  Serial.println(WiFi.macAddress());
  Serial.print("Signal strength: ");
  Serial.println(WiFi.RSSI());
  
  // Solid LED when connected
  digitalWrite(ledPin, LOW);
  
  Serial.println("====================================");
  Serial.println("Setup complete!");
  Serial.println("LED will blink every 2 seconds");
  Serial.println("====================================");
}

void loop() {
  // Check WiFi connection
  if (WiFi.status() == WL_CONNECTED) {
    // Blink LED to show we're alive
    digitalWrite(ledPin, HIGH); // Turn off LED
    delay(1900);
    digitalWrite(ledPin, LOW);  // Turn on LED
    delay(100);
    
    // Print status every 10 seconds
    static unsigned long lastPrint = 0;
    if (millis() - lastPrint > 10000) {
      Serial.println("Status: Connected to " + String(ssid));
      Serial.println("IP: " + WiFi.localIP().toString());
      Serial.println("RSSI: " + String(WiFi.RSSI()) + " dBm");
      Serial.println("Uptime: " + String(millis()/1000) + " seconds");
      Serial.println("Free heap: " + String(ESP.getFreeHeap()) + " bytes");
      Serial.println("---");
      lastPrint = millis();
    }
  } else {
    // Fast blink if disconnected
    digitalWrite(ledPin, LOW);
    delay(100);
    digitalWrite(ledPin, HIGH);
    delay(100);
    Serial.println("WiFi disconnected! Attempting to reconnect...");
    WiFi.begin(ssid, password);
  }
}
