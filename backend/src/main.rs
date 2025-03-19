use crate::db::*;
use crate::error::AppError;
use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};
use clap::Parser;
use common::models::*;
use diesel_async::{pooled_connection::bb8::Pool, AsyncMysqlConnection};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

mod db;
mod error;
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
    connection_pool: Pool<AsyncMysqlConnection>,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }

    let state = AppState {
        connection_pool: get_connection_pool().await,
    };

    tracing_subscriber::fmt::init();
    let index_path = PathBuf::from(&opt.static_dir).join("index.html");
    let app = Router::new()
        .route("/api/speakers", get(speakers))
        .route("/api/breakdown/{type}", put(breakdown))
        .route("/api/population", put(population))
        .route("/api/speeches/{breakdown}/{id}", put(speeches))
        .with_state(state)
        .fallback_service(
            ServeDir::new(&opt.static_dir).not_found_service(ServeFile::new(index_path)),
        )
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let mut port = opt.port;
    if let Ok(port_env) = std::env::var("PORT") {
        port = port_env
            .parse::<u16>()
            .expect("couldn't parse port into u16");
    }

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        port,
    ));

    log::info!("listening on http://{}", sock_addr);

    let listener = tokio::net::TcpListener::bind(&sock_addr)
        .await
        .expect("unable to bind listener");
    axum::serve(listener, app)
        .await
        .expect("Unable to start server");
}

async fn speakers(State(state): State<AppState>) -> Result<Json<Vec<SpeakerResponse>>, AppError> {
    let mut conn = state.connection_pool.get().await?;
    Ok(Json(get_speakers(&mut conn).await?))
}

async fn breakdown(
    State(state): State<AppState>,
    Path(breakdown_type): Path<String>,
    Json(payload): Json<DataRequest>,
) -> Result<Json<Vec<BreakdownResponse>>, AppError> {
    let mut conn = state.connection_pool.get().await?;
    let search = payload.search.to_lowercase().replace(
        |c: char| !(c.is_ascii_alphanumeric() || c == ' ' || c == '-'),
        "",
    );
    let breakdown_type = BreakdownType::from_str(breakdown_type.as_str())?;

    Ok(Json(
        get_breakdown_word_count(&mut conn, breakdown_type, &search).await?,
    ))
}

async fn population(
    State(state): State<AppState>,
    Json(payload): Json<DataRequest>,
) -> Result<Json<Vec<PopulationResponse>>, AppError> {
    let mut conn = state.connection_pool.get().await?;
    let search = payload.search.to_lowercase().replace(
        |c: char| !(c.is_ascii_alphanumeric() || c == ' ' || c == '-'),
        "",
    );

    Ok(Json(get_population_word_count(&mut conn, &search).await?))
}

async fn speeches(
    Path((breakdown_type, id)): Path<(String, i32)>,
    State(state): State<AppState>,
    Json(payload): Json<DataRequest>,
) -> Result<Json<Vec<SpeechResponse>>, AppError> {
    let mut conn = state.connection_pool.get().await?;
    let breakdown_type = BreakdownType::from_str(breakdown_type.as_str())?;
    let search = payload.search.to_lowercase().replace(
        |c: char| !(c.is_ascii_alphanumeric() || c == ' ' || c == '-'),
        "",
    );

    Ok(Json(
        get_speeches(&mut conn, breakdown_type, id, &search).await?,
    ))
}
