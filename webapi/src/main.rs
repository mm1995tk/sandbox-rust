use std::{error::Error, net::SocketAddr, time::Duration};

use axum::{
    extract,
    http::{HeaderValue, StatusCode},
    middleware,
    response::{Html, IntoResponse, Response},
    routing, Router,
};
use axum_extra::extract::CookieJar;
use serde_json::Value;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer};
use ulid::Ulid;
use webapi::{
    db,
    framework::{
        self,
        env::Env,
        logger::{Logger, LoggerInterface},
        session::{mk_cookie, Session},
        AppState, ReqScopedState,
    },
    openapi::example_route,
    openid_connect,
    settings::{CORS_ALLOWED_ORIGINS, SESSION_ID_KEY, TIMEOUT_DURATION},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let env = Env::new();
    let db_client = db::connect(&env.db_url).await;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    let discovery_json =
        reqwest::get("https://accounts.google.com/.well-known/openid-configuration")
            .await?
            .json::<Value>()
            .await?;

    let shared_state = AppState {
        db_client,
        env,
        discovery_json,
    };

    let router = mk_router(shared_state)
        .await
        .into_make_service_with_connect_info::<SocketAddr>();

    axum::serve(listener, router).await?;

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
        .layer(TimeoutLayer::new(Duration::from_secs(TIMEOUT_DURATION)))
        .layer(middleware::from_fn(log))
        .layer(middleware::from_fn(setup))
        .layer(mk_cors_layer())
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

    let req_scoped_state = ReqScopedState::new(req_id);
    let logger = Logger::new(&req_scoped_state, &req, remote_addr);

    if let Some(session_id) = jar.get(SESSION_ID_KEY).map(|c| c.value()) {
        if let Some(session) = framework::session::find_session(session_id).await {
            req.extensions_mut().insert(session);
        }
    }

    req.extensions_mut().insert(req_scoped_state);
    req.extensions_mut().insert(logger);

    Ok(next.run(req).await)
}

async fn log(req: extract::Request, next: middleware::Next) -> Result<Response, StatusCode> {
    let logger = &req
        .extensions()
        .get::<Logger>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .to_owned();

    logger.info("hi");
    let r = next.run(req).await;
    logger.info("bye");

    Ok(r)
}

async fn auth(req: extract::Request, next: middleware::Next) -> Result<Response, StatusCode> {
    if let Some(session) = req.extensions().get::<Session>() {
        let c = mk_cookie(session.session_id.to_string());
        let jar = CookieJar::from_headers(req.headers()).add(c);
        Ok((jar, next.run(req).await).into_response())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

fn mk_cors_layer() -> CorsLayer {
    CorsLayer::new().allow_origin(CORS_ALLOWED_ORIGINS.map(|origin| {
        origin
            .parse::<HeaderValue>()
            .expect("originをパースできる必要がある")
    }))
}
