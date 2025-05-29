use actix_web::HttpResponse;
use core::{User, HttpSendResponse};

pub async fn post() -> HttpResponse {
    println!("Ping request received!");
    
    let user = User::new(
        1,
        "Doe".to_string(),
        "john.doe@example.com".to_string(),
    );
    
    let message = format!("Bienvenue {} {} !", user.name, user.email);
    let user_value = serde_json::to_value(&user).unwrap();
    
    HttpResponse::Ok().json(HttpSendResponse {
        status: 200,
        message: Some(message),
        data: Some(user_value),
    })
}
