use axum::{
    extract,
    http::StatusCode,
    middleware,
    response::{self, Response},
    Router,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
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

    if let Err(e) = axum::serve(listener, mk_router()).await {
        println!("{}", e);
    }
}

fn mk_router() -> Router {
    let shared_state = AppState;

    Router::new()
        .nest(
            example_route::PATH,
            example_route::mk_router().route_layer(middleware::from_fn(auth)),
        )
        .nest(openid_connect::PATH, openid_connect::mk_router())
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
