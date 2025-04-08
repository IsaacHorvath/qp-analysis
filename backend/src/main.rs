//! The backend package for the house words app.
//!
//! Serves the frontend wasm binary, and provides api routes that return the results
//! of SQL queries on an external MariaDB instance.

use crate::db::get_connection_pool;
use crate::handlers::*;
use crate::reaper::reaper;
use axum::{
    routing::{get, put},
    Router,
};
use clap::Parser;
use diesel_async::{pooled_connection::bb8::Pool, AsyncMysqlConnection};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

use crate::reaper::Message;
use tokio::sync::{mpsc, mpsc::Sender};
use tokio::time::{Duration, sleep};
use tower::ServiceBuilder;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::EnvFilter;

mod db;
mod dummy_db;
mod error;
mod handlers;
mod reaper;

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

    #[clap(long = "log-dir", default_value = "./logs")]
    log_dir: String,

    #[clap(short, long, default_value_t = false)]
    dummy: bool,
}

/// A struct to store the global backend state (database connection pool and mpsc
/// channel sender).

#[derive(Clone)]
struct AppState {
    /// The connection pool for the app. If None, we are in dummy mode and pulling
    /// hardcoded demo data. If Some, the pool will retain 10 idle connections and
    /// will open at max 50 connections.
    connection_pool: Option<Pool<AsyncMysqlConnection>>,

    /// A sender to send registration and kill messages to the reaper.
    sender: Sender<Message>,
}

/// The main backend function.
///
/// This function parses the command line arguments; sets up the state, the reaper,
/// and its message channel; configures all the routes and the tracing middleware;
/// sets up the listener and serves it via axum.

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", opt.log_level);
    }

    let (sender, mut receiver) = mpsc::channel(50);

    let state = AppState {
        connection_pool: if opt.dummy {
            None
        } else {
            Some(get_connection_pool().await)
        },
        sender,
    };

    if !opt.dummy {
        let state = state.clone();
        tokio::spawn(async move {
            reaper(state.connection_pool.unwrap(), &mut receiver).await;
        });
    }

    let file_appender = tracing_appender::rolling::hourly(opt.log_dir, "prefix.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .with_writer(non_blocking)
        .init();

    let governor_conf = std::sync::Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(5)
            .finish()
            .expect("couldn't build rate limit governor"),
    );
    let governor_limiter = governor_conf.limiter().clone();

    let service = ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().include_headers(true)),
        )
        .layer(GovernorLayer {
            config: governor_conf,
        });

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(60)).await;
            tracing::trace!("rate limiting storage size: {}", governor_limiter.len());
            governor_limiter.retain_recent();
        }
    });

    let index_path = PathBuf::from(&opt.static_dir).join("index.html");
    let app = Router::new()
        .route("/api/speakers", get(speakers))
        .route("/api/breakdown/{type}", put(breakdown))
        .route("/api/population", put(population))
        .route("/api/speeches/{breakdown}/{id}", put(speeches))
        .route("/api/cancel", put(cancel))
        .route("/api/cancel/speeches", put(cancel_speech))
        .with_state(state)
        .fallback_service(
            ServeDir::new(&opt.static_dir).not_found_service(ServeFile::new(index_path)),
        )
        .layer(service)
        .into_make_service_with_connect_info::<SocketAddr>();

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

    let listener = tokio::net::TcpListener::bind(&sock_addr)
        .await
        .expect("unable to bind listener");

    axum::serve(listener, app)
        .await
        .expect("Unable to start server");
}
