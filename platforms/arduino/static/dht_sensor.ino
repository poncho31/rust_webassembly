// DHT22 Temperature and Humidity Sensor Example
// Connect DHT22 to pin 2

#include <DHT.h>

#define DHT_PIN 2
#define DHT_TYPE DHT22

DHT dht(DHT_PIN, DHT_TYPE);

void setup() {
  Serial.begin(9600);
  Serial.println("DHT22 Temperature & Humidity Sensor");
  Serial.println("===================================");
  
  dht.begin();
}

void loop() {
  delay(2000);  // Wait 2 seconds between readings
  
  float humidity = dht.readHumidity();
  float temperature = dht.readTemperature();
  
  if (isnan(humidity) || isnan(temperature)) {
    Serial.println("Failed to read from DHT sensor!");
    return;
  }
  
  Serial.print("Humidity: ");
  Serial.print(humidity);
  Serial.print("% | Temperature: ");
  Serial.print(temperature);
  Serial.println("°C");
}
