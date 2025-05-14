use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use core::http::config::config;
use env_logger::Env;
use sea_orm::{Database, DatabaseConnection};

#[actix_web::main]
async fn main() -> std::io::Result<()>  {
    // Initialize logging
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get environment variables
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let host_url = std::env::var("HOST_URL").expect("HOST_URL must be set");
    let host_port = std::env::var("HOST_PORT").expect("HOST_PORT must be set");

    // Connect to the database
    let conn: DatabaseConnection = Database::connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin() // Allow requests from any origin (use carefully in production)
                    .allow_any_method() // Allow any HTTP method (GET, POST, etc.)
                    .allow_any_header() // Allow any headers
            )
            .app_data(web::Data::new(conn.clone())) // Pass the database connection
            .configure(config) // Configure the routes and services
    })
        .bind(format!("{host_url}:{host_port}"))?
        .run()
        .await
}
