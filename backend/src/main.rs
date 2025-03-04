use axum::{
    routing::{put, get},
    extract::Path,
    Router,
    Json
};
use clap::Parser;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use std::path::PathBuf;
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use crate::db::*;
use common::models::*;

mod db;

#[derive(Parser, Debug)]
#[clap(name = "backend", about = "queens park analysis backend")]
struct Opt {
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,
    
    #[clap(short = 'a', long = "addr", default_value = "::1")]
    addr: String,

    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,
    
    #[clap(long = "static-dir", default_value = "./dist")]
    static_dir: String,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();
    
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }
    tracing_subscriber::fmt::init();
    let index_path = PathBuf::from(&opt.static_dir).join("index.html");
    let app = Router::new()
        .route("/api/speakers", get(speakers))
        .route("/api/breakdown/{type}", put(breakdown))
        .route("/api/speeches/{breakdown}/{id}", put(speeches))
        .fallback_service(ServeDir::new(&opt.static_dir).not_found_service(ServeFile::new(index_path)))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let mut port = opt.port;
    if let Ok(port_env) = std::env::var("PORT") {
        port = port_env.parse::<u16>().unwrap();
    }
        
    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        port,
    ));

    log::info!("listening on http://{}", sock_addr);

    let listener = tokio::net::TcpListener::bind(&sock_addr).await.unwrap();
    axum::serve(listener, app).await.expect("Unable to start server");
}

async fn speakers() -> Json<Vec<SpeakerResponse>> {
    let mut connection = establish_connection(); //todo: stop reopening and closing this?
    Json(get_speakers(&mut connection))
}

async fn breakdown(Path(breakdown_type): Path<String>, Json(payload): Json<DataRequest>) -> Json<Option<Vec<BreakdownResponse>>> {
    let mut connection = establish_connection();
    let breakdown_type = BreakdownType::from_str(breakdown_type.as_str())
        .expect(format!("couldn't process breakdown type {}", breakdown_type).as_str());
    let search = payload.search
        .to_lowercase()
        .replace(|c: char| !(c.is_ascii_alphanumeric() || c == ' ' || c == '-'), "");

    Json(get_breakdown_word_count(&mut connection, breakdown_type, &search))
}

async fn speeches(Path((breakdown_type, id)): Path<(String, i32)>, Json(payload): Json<DataRequest>) -> Json<Vec<SpeechResponse>> {
    let mut connection = establish_connection();
    let breakdown_type = BreakdownType::from_str(breakdown_type.as_str())
        .expect(format!("couldn't process breakdown type {}", breakdown_type).as_str());
    let search = payload.search
        .to_lowercase()
        .replace(|c: char| !(c.is_ascii_alphanumeric() || c == ' ' || c == '-'), "");

    Json(get_speeches(&mut connection, breakdown_type, id, &search))
}
