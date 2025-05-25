use actix_files::Files;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, error::ErrorInternalServerError};
use core::User;
use serde::{Serialize, Deserialize};
use sqlx::{postgres::PgPoolOptions, PgPool, Pool, Postgres, Row};
use anyhow::Error;
use dotenv::dotenv;
use std::env;
use uuid::Uuid;
use actix_cors::Cors;
use actix_web::http::header;

#[derive(Deserialize)]
struct CreateUser {
    name:  String,
    email: String,
}

#[derive(Serialize)]
struct ResponseUser {
    id:    String,
    name:  String,
    email: String,
}

impl From<User> for ResponseUser {
    fn from(u: User) -> Self {
        ResponseUser { id: u.id, name: u.name, email: u.email }
    }
}

async fn create_database() -> Result<(), Error> {
    let pg_host     = env::var("PG_HOST").expect("PG_HOST must be set");
    let pg_user     = env::var("PG_USER").expect("PG_USER must be set");
    let pg_password = env::var("PG_PASSWORD").expect("PG_PASSWORD must be set");
    let pg_database = env::var("PG_DATABASE").expect("PG_DATABASE must be set");
    
    // Connexion à la base postgres par défaut pour créer notre base
    let postgres_url = format!(
        "postgres://{}:{}@{}/postgres",
        pg_user, pg_password, pg_host
    );

    match PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgres_url)
        .await {
            Ok(pool) => {
                // Tentative de création de la base de données
                let query = format!("CREATE DATABASE {}", pg_database);
                match sqlx::query(&query).execute(&pool).await {
                    Ok(_)  => println!("Database created successfully"),
                    Err(e) => println!("Database creation failed (might already exist): {}", e),
                }
                Ok(())
            },
            Err(e) => {
                println!("Could not connect to postgres database: {}", e);
                Err(Error::msg("Database connection failed"))
            }
    }
}

async fn init_db() -> Result<Pool<Postgres>, Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    println!("Ensuring database exists...");

    // Tenter de créer la base de données si elle n'existe pas
    if let Err(e) = create_database().await {
        println!("Warning: Could not create database: {}", e);
    }

    println!("Connecting to database...");
    
    // Tenter la connexion plusieurs fois avec un délai
    for i in 1..=3 {
        match PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await {
                Ok(pool) => {
                    // Test de connexion
                    if let Ok(_) = sqlx::query("SELECT 1").execute(&pool).await {
                        println!("Database connection successful!");
                        return Ok(pool);
                    }
                },
                Err(e) => {
                    println!("Connection attempt {} failed: {}", i, e);
                    if i < 3 {
                        println!("Retrying in 2 seconds...");
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                }
            }
    }

    Err(Error::msg("Could not connect to database after 3 attempts"))
}

#[get("/api/users")]
async fn list_users(pool: web::Data<PgPool>) -> Result<HttpResponse, actix_web::Error> {
    println!("Fetching users...");
    
    let result = sqlx::query(
        "SELECT id::text, name, email FROM users"
    )
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let users: Vec<User> = rows
                .iter()
                .map(|row| User {
                    id: row.try_get(0).unwrap_or_default(),
                    name: row.try_get(1).unwrap_or_default(),
                    email: row.try_get(2).unwrap_or_default(),
                })
                .collect();
            Ok(HttpResponse::Ok().json(users))
        }
        Err(e) => {
            println!("Error fetching users: {:?}", e);
            Ok(HttpResponse::Ok().json(Vec::<User>::new()))
        }
    }
}

#[post("/api/users")]
async fn add_user(
    pool: web::Data<PgPool>,
    form: web::Json<CreateUser>
) -> Result<HttpResponse, actix_web::Error> {
    println!("Adding new user...");
    let id = Uuid::new_v4();
    
    // Create table if not exists
    if let Err(e) = sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE
        )"
    )
    .execute(pool.get_ref())
    .await {
        println!("Error creating table: {:?}", e);
        return Err(ErrorInternalServerError("Database error"));
    }

    // Insert new user
    match sqlx::query(
        "INSERT INTO users (id, name, email) VALUES ($1, $2, $3) RETURNING id::text, name, email"
    )
    .bind(id)
    .bind(&form.name)
    .bind(&form.email)
    .fetch_one(pool.get_ref())
    .await {
        Ok(row) => {
            let user = User {
                id: row.try_get(0).unwrap_or_default(),
                name: row.try_get(1).unwrap_or_default(),
                email: row.try_get(2).unwrap_or_default(),
            };
            Ok(HttpResponse::Created().json(ResponseUser::from(user)))
        }
        Err(e) => {
            println!("Error inserting user: {:?}", e);
            Err(ErrorInternalServerError("Could not create user"))
        }
    }
}

// Modifier main pour une meilleure gestion des erreurs
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8088".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a number");

    match init_db().await {
        Ok(pool) => {
            let static_path = std::env::current_dir()?.join("client").join("static");
            let pkg_path = std::env::current_dir()?.join("client").join("static").join("pkg");
            
            println!("Static files path: {:?}", static_path);
            println!("WASM pkg path: {:?}", pkg_path);
            
            assert!(static_path.exists(), "Static directory not found!");
            assert!(pkg_path.exists(), "WASM pkg directory not found!");
            assert!(static_path.join("index.html").exists(), "index.html not found!");

            HttpServer::new(move || {
                let cors = Cors::permissive()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .expose_headers(vec![
                        "content-type",
                        "content-length",
                        "accept",
                    ])
                    .max_age(3600);

                App::new()
                    .wrap(actix_web::middleware::Logger::default())
                    .wrap(cors)
                    .app_data(web::Data::new(pool.clone()))
                    .service(web::scope("/api")
                        .service(list_users)
                        .service(add_user)
                    )
                    .service(
                        Files::new("/pkg", &pkg_path)
                            .show_files_listing()
                    )
                    .service(
                        Files::new("/", &static_path)
                            .index_file("index.html")
                    )
                    .default_service(web::route().to(|| async {
                        HttpResponse::NotFound().body("Not Found")
                    }))
            })
            .workers(1)
            .bind((host, port))?
            .run()
            .await
        },
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    }
}
