// Arduino LED Blink Example
// Simple example to test Arduino deployment

void setup() {
  // Initialize digital pin LED_BUILTIN as an output
  pinMode(LED_BUILTIN, OUTPUT);
  
  // Initialize serial communication
  Serial.begin(9600);
  Serial.println("Arduino LED Blink Example Started!");
}

void loop() {
  digitalWrite(LED_BUILTIN, HIGH);   // Turn the LED on
  Serial.println("LED ON");
  delay(1000);                       // Wait for a second
  
  digitalWrite(LED_BUILTIN, LOW);    // Turn the LED off
  Serial.println("LED OFF");
  delay(1000);                       // Wait for a second
}
