use std::{error::Error, net::SocketAddr, time::Duration};

use axum::{
    extract,
    http::StatusCode,
    middleware,
    response::{Html, Response},
    routing, Router,
};
use axum_extra::extract::CookieJar;
use serde_json::Value;
use ulid::Ulid;
use webapi::{
    db,
    framework::{self, env::Env, logger::LoggerInterface, AppState, ReqScopedState},
    openapi::example_route,
    openid_connect,
    settings::SESSION_ID_KEY,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let env = Env::new();
    let db_client = db::connect(&env.db_url);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000");
    let discovery_json =
        reqwest::get("https://accounts.google.com/.well-known/openid-configuration")
            .await?
            .json::<Value>()
            .await?;

    let shared_state = AppState {
        db_client: db_client.await,
        env,
        discovery_json,
    };

    let router = mk_router(shared_state)
        .await
        .into_make_service_with_connect_info::<SocketAddr>();

    axum::serve(listener.await?, router).await?;

    Ok(())
}

async fn mk_router(shared_state: AppState) -> Router {
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

async fn setup(mut req: extract::Request, next: middleware::Next) -> Result<Response, StatusCode> {
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
        req_scoped_state.session = framework::session::find_session(session_id).await;
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
