use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use axum::{
    routing::{put, get},
    extract::{Path, State},
    Router,
    Json
};
use clap::Parser;
use diesel::MysqlConnection;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use std::path::PathBuf;
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use crate::dummy_db::*;
use crate::error::AppError;
use common::models::*;

mod error;
mod dummy_db;
// todo use clap for dev dummy db

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

#[derive(Clone)]
struct AppState {
    connection_pool: Pool<ConnectionManager<MysqlConnection>>,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();
    
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }
    
    let state = AppState { connection_pool: get_connection_pool() };
    
    tracing_subscriber::fmt::init();
    let index_path = PathBuf::from(&opt.static_dir).join("index.html");
    let app = Router::new()
        .route("/api/speakers", get(speakers))
        .route("/api/breakdown/{type}", put(breakdown))
        .route("/api/population", put(population))
        .route("/api/speeches/{breakdown}/{id}", put(speeches))
        .with_state(state)
        .fallback_service(ServeDir::new(&opt.static_dir).not_found_service(ServeFile::new(index_path)))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let mut port = opt.port;
    if let Ok(port_env) = std::env::var("PORT") {
        port = port_env.parse::<u16>().expect("couldn't parse port into u16");
    }
        
    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        port,
    ));

    log::info!("listening on http://{}", sock_addr);

    let listener = tokio::net::TcpListener::bind(&sock_addr).await.unwrap();
    axum::serve(listener, app).await.expect("Unable to start server");
}

fn connect<T, F>(state: AppState, f: F) -> Result<Json<Vec<T>>, AppError> where F: Fn(&mut MysqlConnection) -> Result<Json<Vec<T>>, AppError> {
    match state.connection_pool.get() {
        Ok(mut conn) => f(&mut conn),
        Err(_) => Err(AppError::ConnectionPoolError),
    }
}

async fn speakers(State(state): State<AppState>) -> Result<Json<Vec<SpeakerResponse>>, AppError> {
    connect(state, |conn| { Ok(Json(get_speakers(conn))) })
}

async fn breakdown(State(state): State<AppState>, Path(breakdown_type): Path<String>, Json(payload): Json<DataRequest>) -> Result<Json<Vec<BreakdownResponse>>, AppError> {
    connect(state, |conn| -> Result<Json<Vec<BreakdownResponse>>, AppError> {
        let breakdown_type = BreakdownType::from_str(breakdown_type.as_str())?;
        let search = payload.search
            .to_lowercase()
            .replace(|c: char| !(c.is_ascii_alphanumeric() || c == ' ' || c == '-'), "");

        Ok(Json(get_breakdown_word_count(conn, breakdown_type, &search)))
    })
}

async fn population(State(state): State<AppState>, Json(payload): Json<DataRequest>) -> Result<Json<Vec<PopulationResponse>>, AppError> {
    connect(state, |conn| -> Result<Json<Vec<PopulationResponse>>, AppError> {
        let search = payload.search
            .to_lowercase()
            .replace(|c: char| !(c.is_ascii_alphanumeric() || c == ' ' || c == '-'), "");

        Ok(Json(get_population_word_count(conn, &search)))
    })
}

async fn speeches(Path((breakdown_type, id)): Path<(String, i32)>, State(state): State<AppState>, Json(payload): Json<DataRequest>) -> Result<Json<Vec<SpeechResponse>>, AppError> {
    connect(state, |conn| -> Result<Json<Vec<SpeechResponse>>, AppError> {
        let breakdown_type = BreakdownType::from_str(breakdown_type.as_str())?;
        let search = payload.search
            .to_lowercase()
            .replace(|c: char| !(c.is_ascii_alphanumeric() || c == ' ' || c == '-'), "");

        Ok(Json(get_speeches(conn, breakdown_type, id, &search)))
    })
}
