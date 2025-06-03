use actix_web::{web, HttpResponse, HttpRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct WeatherQuery {
    pub region: Option<String>,
    pub pays: Option<String>,
}

#[derive(Serialize)]
pub struct WeatherResponse {
    pub temperature: String,
    pub region: String,
    pub status: u16,
    pub message: String,
}

/// API pour récupérer la température d'une région
pub async fn get_temperature(query: web::Query<WeatherQuery>, _req: HttpRequest) -> HttpResponse {
    let region = query.region.clone().unwrap_or_else(|| "not found".to_string());
    let pays   = query.pays.clone().unwrap_or_else(|| "not found".to_string());
    
    println!(" query {:?}", query);


    // Simulation de données météo pour différentes villes
    let weather_data = get_mock_weather_data();
    
    let temperature = weather_data.get(&region.to_lowercase())
        .copied()
        .unwrap_or_else(|| {
            // Si la ville n'est pas trouvée, retourner une température aléatoire
            simulate_random_temperature()
        });

    let response = WeatherResponse {
        temperature : format!("{}°C, {}", temperature.to_string(), &pays),
        region: region.clone(),
        status: 200,
        message: format!("Température actuelle pour {} {}", region , pays),
    };

    println!("🌡️ Weather API - Région: {}, Température: {}°C", region, temperature);
    
    HttpResponse::Ok().json(response)
}

/// Données météo fictives pour différentes villes
fn get_mock_weather_data() -> HashMap<String, f32> {
    let mut weather = HashMap::new();
    
    // Villes belges
    weather.insert("bruxelles".to_string(), 18.5);
    weather.insert("anvers".to_string(), 17.8);
    weather.insert("gand".to_string(), 16.9);
    weather.insert("liège".to_string(), 19.2);
    weather.insert("charleroi".to_string(), 17.1);
    
    // Villes européennes
    weather.insert("paris".to_string(), 21.3);
    weather.insert("london".to_string(), 15.7);
    weather.insert("berlin".to_string(), 16.4);
    weather.insert("amsterdam".to_string(), 17.2);
    weather.insert("madrid".to_string(), 25.8);
    weather.insert("rome".to_string(), 23.9);
    
    // Villes du monde
    weather.insert("new york".to_string(), 22.1);
    weather.insert("tokyo".to_string(), 24.6);
    weather.insert("sydney".to_string(), 19.8);
    weather.insert("montreal".to_string(), 14.3);
    
    weather
}

/// Simule une température aléatoire pour les villes non reconnues
fn simulate_random_temperature() -> f32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    let hash = hasher.finish();
    
    // Température entre 10°C et 30°C basée sur le hash
    10.0 + ((hash % 200) as f32 / 10.0)
}
