use axum::routing::get;
use tower_http::cors::{Any, CorsLayer};

use dotenv;

mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    dotenv::dotenv().ok();

    // Cross-Origin Resource Sharing
    let cors = CorsLayer::new().allow_origin(Any);
    
    // Get the Axum port
    let port: u16 = match dotenv::var("PORT") {
        Ok(port) => match str::parse::<u16>(port.as_str()) {
            Ok(port) => port,
            Err(err) => {
                eprintln!("Could not parse port number: {}", err);
                println!("Defaulting to port 8000");
                8000
            }
        },
        Err(_) => {
            println!("Defaulting to port 8000");
            8000
        }
    };
    
    // Implement the File Handler
    let file_router = axum::Router::new()
        .route("/", get(routes::serve_index))
        .route("/*path", get(routes::serve_file));

    // Implement the Authentication Handler
    let auth_router = axum::Router::new()
        .route("/", get(routes::serve_index))
        .route("/login", get(routes::serve_index))
        .route("/register", get(routes::serve_index));

    // Implement the User Handler
    let user_router = axum::Router::new()
        .route("/", get(routes::serve_index))
        .route("/:id", get(routes::serve_index))
        .route("/:id/follow", get(routes::serve_index))
        .route("/:id/unfollow", get(routes::serve_index));

    // Integrate the Posts Handler
    let posts_router = axum::Router::new()
        .route("/", get(routes::serve_index))
        .route("/:id", get(routes::serve_index))
        .route("/:id/like", get(routes::serve_index))
        .route("/:id/timeline", get(routes::serve_index));

    // Integrate the Upload Handler
    let upload_router = axum::Router::new()
        .route("/", get(routes::serve_index));

    // Integrate React CSR with the File Handler
    let react_csr_fs_router = axum::Router::new()
        .route("/home", get(routes::serve_index))
        .route("/profile/:id", get(routes::serve_index))
        .route("/chat", get(routes::serve_index))
        .nest_service("/auth", auth_router)
        .nest_service("/user", user_router)
        .nest_service("/posts", posts_router)
        .nest_service("/upload", upload_router)
        .nest_service("/", file_router);
        
    // Create the main router (DO NOT include a state)
    let app = axum::Router::new()
        .nest_service("/", react_csr_fs_router)
        .layer(cors)
        .fallback(routes::page_not_found);

    // Create a Socket Address
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    println!("Server starting on http://{}", addr);
    
    // Spawn server with Axum
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to start server");

    Ok(())
}