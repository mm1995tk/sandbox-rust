use std::time::Duration;

use axum::{
    extract,
    http::StatusCode,
    middleware,
    response::{self, Html, Response},
    routing, Router,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use ulid::Ulid;
use webapi::{
    framework::{self, AppState, ReqScopedState},
    openapi::example_route,
    openid_connect,
    settings::SESSION_ID_KEY,
};

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("server can run");

    let router = mk_router(connect_db().await);
    if let Err(e) = axum::serve(listener, router).await {
        println!("{}", e);
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

fn mk_router(db_client: DatabaseConnection) -> Router {
    let shared_state = AppState { db_client };

    Router::new()
        .nest(
            example_route::PATH,
            example_route::mk_router().route_layer(middleware::from_fn(auth)),
        )
        .nest(openid_connect::PATH, openid_connect::mk_router())
        .route(
            "/login",
            routing::get(|| async {
                Html(
                    "<html>

                <div>
                  <a href='http://localhost:3000/openid-connect'>
                    google でログイン
                  </a>
                </div>
                
                </html>",
                )
            }),
        )
        .layer(middleware::from_fn(log))
        .layer(middleware::from_fn(setup))
        .with_state(shared_state)
}

async fn setup(
    mut req: extract::Request,
    next: middleware::Next,
) -> Result<response::Response, StatusCode> {
    // CookieJar => クッキー缶　=> クッキーがいっぱい入っている => 他言語だとCookiesみたいなやつ
    let jar = CookieJar::from_headers(req.headers());
    let req_id: Ulid = Ulid::new();

    let mut req_scoped_state = ReqScopedState::new(req_id, None);

    if let Some(session_id) = jar.get(SESSION_ID_KEY).map(|c| c.value()) {
        req_scoped_state.session = framework::find_session(session_id).await;
    }

    req.extensions_mut().insert(req_scoped_state);

    Ok(next.run(req).await)
}

async fn log(req: extract::Request, next: middleware::Next) -> Result<Response, StatusCode> {
    let item = req
        .extensions()
        .get::<ReqScopedState>()
        .expect("setup is already done.");

    println!("hi, {}", item.ts);
    let r = next.run(req).await;
    println!("bye, {}", Utc::now());
    return Ok(r);
}

async fn auth(req: extract::Request, next: middleware::Next) -> Result<Response, StatusCode> {
    let item = req
        .extensions()
        .get::<ReqScopedState>()
        .expect("setup is already done.");

    if item.session.is_some() {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
