mod database;
mod handlers;
mod middleware;
mod models;
mod routes;

use actix_web::{App, HttpServer};
use middleware::auth::AuthMiddleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // Establish database connection
    let pool = database::establish_connection()
        .await
        .expect("Failed to connect to database");

    println!("Server running at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(pool.clone()))
            .wrap(AuthMiddleware::new(pool.clone()))
            .configure(routes::config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
