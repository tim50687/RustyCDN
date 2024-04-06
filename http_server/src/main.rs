mod util;

use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use awc::http::StatusCode;
use clap::Parser;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use util::cache_system::CacheSystem;
use util::cl_parser::Cli;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref CACHE: Arc<Mutex<CacheSystem>> = Arc::new(Mutex::new(CacheSystem::new(18_000_000)));
}

struct AppState {
    origin: String,
}

#[get("/{content_path}")]
async fn serve_content(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let content_path: String = req
        .match_info()
        .get("content_path")
        .unwrap()
        .parse()
        .unwrap();

    dbg!(&state.origin);

    if CACHE.lock().await.contains_key(&content_path) {
        dbg!("cache!!!");

        HttpResponse::Ok().body(CACHE.lock().await.get(&content_path))
    } else {
        let client = awc::Client::default();
        let response = client
            .get(format!("http://{}:8080/{}", state.origin, content_path)) // <- Create request builder
            .insert_header(("Accept-Encoding", "gzip"))
            .insert_header(("User-Agent", "Actix-web"))
            .send() // <- Send http request
            .await;

        match response {
            Ok(mut res) => match res.status() {
                StatusCode::OK => {
                    let body_bytes = res.body().limit(20_000_000).await.expect("failed!!!");
                    let body = String::from_utf8_lossy(&body_bytes).to_string();
                    CACHE.lock().await.add(&content_path, &body);
                    HttpResponse::Ok().body(body)
                }
                _ => HttpResponse::NotFound().body(""),
            },
            Err(_) => HttpResponse::NotFound().body(""),
        }
    }
}

#[get("/grading/beacon")]
async fn respond_beacon() -> impl Responder {
    HttpResponse::NoContent()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    dbg!(&cli);

    let app_state = web::Data::new(AppState { origin: cli.origin });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(respond_beacon)
            .service(serve_content)
    })
    .keep_alive(Duration::from_secs(25))
    .bind(("0.0.0.0", cli.port))?
    .run()
    .await
}
