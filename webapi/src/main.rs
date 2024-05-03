use std::{error::Error, net::SocketAddr, time::Duration};

use axum::{
    extract::{self, ConnectInfo},
    http::StatusCode,
    middleware,
    response::{self, Html, IntoResponse, Response},
    routing, Router,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde_json::Value;
use ulid::Ulid;
use webapi::{
    framework::{self, logger::LoggerInterface, AppState, Env, ReqScopedState},
    openapi::example_route,
    openid_connect,
    settings::SESSION_ID_KEY,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("server can run");
    let discovery_json =
        reqwest::get("https://accounts.google.com/.well-known/openid-configuration")
            .await?
            .json::<Value>()
            .await?;
    let router = mk_router(connect_db().await, discovery_json)
        .into_make_service_with_connect_info::<SocketAddr>();
    if let Err(e) = axum::serve(listener, router).await {
        println!("{}", e);
    }
    Ok(())
}

fn mk_env() -> Env {
    let google_client_id = std::env::var("GOOGLE_CLIENT_ID")
        .expect("環境変数にGOOGLE_CLIENT_IDをセットしてください。");
    let google_client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
        .expect("環境変数にGOOGLE_CLIENT_SECRETをセットしてください。");
    let google_redirect_uri =
        std::env::var("REDIRECT_URI").expect("環境変数にREDIRECT_URIをセットしてください。");

    Env {
        google_client_id,
        google_redirect_uri,
        google_client_secret,
    }
}

async fn connect_db() -> DatabaseConnection {
    let mut opt =
        ConnectOptions::new("postgresql://localhost:5433/postgres?user=postgres&password=postgres");
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);
    // .sqlx_logging_level(log::LevelFilter::Info)
    // .set_schema_search_path("my_schema"); // Setting default PostgreSQL schema

    Database::connect(opt).await.expect("db接続に成功すべき")
}

fn mk_router(db_client: DatabaseConnection, discovery_json: Value) -> Router {
    let shared_state = AppState {
        db_client,
        env: mk_env(),
        discovery_json,
    };

    Router::new()
        .nest(
            example_route::PATH,
            example_route::mk_router().route_layer(middleware::from_fn(auth)),
        )
        .nest(openid_connect::PATH, openid_connect::mk_router())
        .route(
            "/login",
            routing::get(|| async {
                use std::fs;
                let contents = fs::read_to_string("webapi/src/index.html")
                    .expect("Should have been able to read the file");
                Html(contents)
            }),
        )
        .layer(middleware::from_fn(log))
        .layer(middleware::from_fn(setup))
        .with_state(shared_state)
}

async fn setup(
    mut req: extract::Request,
    next: middleware::Next,
    // ConnectInfo(addr): ConnectInfo<SocketAddr>
) -> Result<Response, StatusCode> {
    // CookieJar => クッキー缶　=> クッキーがいっぱい入っている => 他言語だとCookiesみたいなやつ
    let jar = CookieJar::from_headers(req.headers());
    let req_id: Ulid = Ulid::new();

    let remote_addr = &req
        .extensions()
        .get::<extract::ConnectInfo<SocketAddr>>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .0;

    let mut req_scoped_state = ReqScopedState::new(req_id, None, &req, &remote_addr);

    if let Some(session_id) = jar.get(SESSION_ID_KEY).map(|c| c.value()) {
        req_scoped_state.session = framework::find_session(session_id).await;
    }

    req.extensions_mut().insert(req_scoped_state);

    Ok(next.run(req).await)
}

async fn log(req: extract::Request, next: middleware::Next) -> Result<Response, StatusCode> {
    let item = &req
        .extensions()
        .get::<ReqScopedState>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .to_owned();

    let logger = item.logger();

    logger.info("hi");
    let r = next.run(req).await;
    logger.info("bye");

    return Ok(r);
}

async fn auth(req: extract::Request, next: middleware::Next) -> Result<Response, StatusCode> {
    let item = req
        .extensions()
        .get::<ReqScopedState>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if item.session.is_some() {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
