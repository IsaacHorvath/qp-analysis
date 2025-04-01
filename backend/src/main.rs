use crate::db::*;
use crate::error::AppError;
use crate::reaper::reaper;
use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};
use clap::Parser;
use common::models::*;
use diesel_async::{pooled_connection::bb8::Pool, AsyncMysqlConnection};
use reaper::ActiveQuery;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use tokio::sync::{mpsc, mpsc::Sender};
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use crate::reaper::Message;

mod db;
mod error;
mod reaper;

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
    sender: Sender<Message>,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }
    
    let (sender, mut receiver) = mpsc::channel(50);

    let state = AppState {
        connection_pool: get_connection_pool().await,
        sender,
    };
    
    {
        let state = state.clone();
        tokio::spawn(async move { reaper(state.connection_pool, &mut receiver).await; });
    }

    tracing_subscriber::fmt::init();
    let index_path = PathBuf::from(&opt.static_dir).join("index.html");
    let app = Router::new()
        .route("/api/speakers", get(speakers))
        .route("/api/breakdown/{type}", put(breakdown))
        .route("/api/population", put(population))
        .route("/api/speeches/{breakdown}/{id}", put(speeches))
        .route("/api/cancel", put(cancel))
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
    let conn_id = get_connection_id(&mut conn).await?;
    
    let token = CancellationToken::new();
    
    state.sender.send(Message::Register((
        ActiveQuery { uuid: payload.uuid.clone(), conn_id },
        token.clone()
    ))).await?;
    
    let search = payload.search.to_lowercase().replace(
        |c: char| !(c.is_ascii_alphanumeric() || c == ' ' || c == '-'),
        "",
    );
    
    let t = tokio::select! {
        res = get_population_word_count(&mut conn, &search) => {
            Ok(Json(res?))
        }
        _ = token.cancelled() => {
            Err(AppError::Cancelled)
        }
    };
    //let json = Json(get_population_word_count(&mut conn, &search).await?);
    
    state.sender.send(Message::Deregister(
        ActiveQuery { uuid: payload.uuid, conn_id }
    )).await?;
    
    t
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

async fn cancel(State(state): State<AppState>,Json(payload): Json<CancelRequest>) -> Result<(), AppError> {
    state.sender.send(Message::Kill(payload.uuid)).await?;
    Ok(())
}
