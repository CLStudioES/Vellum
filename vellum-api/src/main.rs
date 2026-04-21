mod middleware;
mod models;
mod routes;

use axum::extract::DefaultBodyLimit;
use axum::routing::{delete, get, post, put};
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};

use middleware::JwtSecret;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub jwt_secret: JwtSecret,
}

const MAX_BODY_SIZE: usize = 2 * 1024 * 1024; // 2MB

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Neon");

    let state = AppState {
        db: pool,
        jwt_secret: JwtSecret(jwt_secret.clone()),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login))
        .route("/projects", get(routes::projects::list_projects))
        .route("/projects", post(routes::projects::create_project))
        .route("/projects/{id}", delete(routes::projects::delete_project))
        .route("/projects/{id}/members", get(routes::members::list_members))
        .route("/projects/{id}/members", post(routes::members::invite_member))
        .route("/projects/{id}/members/{user_id}", delete(routes::members::remove_member))
        .route("/projects/{id}/entries", get(routes::entries::list_entries))
        .route("/projects/{id}/entries", put(routes::entries::upsert_entries))
        .layer(DefaultBodyLimit::max(MAX_BODY_SIZE))
        .layer(axum::Extension(JwtSecret(jwt_secret)))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind");

    println!("Vellum API running on port {}", port);
    axum::serve(listener, app).await.expect("Server failed");
}
