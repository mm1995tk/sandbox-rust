use axum::{extract, http::StatusCode, middleware, response, Router};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use ulid::Ulid;
use webapi::{
    framework::{self, ReqScopedState},
    openapi::example_route,
};

#[tokio::main]
async fn main() {
    match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Err(e) => {
            println!("{}", e);
        }
        Ok(listener) => {
            if let Err(e) = axum::serve(listener, mk_router()).await {
                println!("{}", e);
            }
        }
    }
}

fn mk_router() -> Router {
    let shared_state = framework::AppState;

    Router::new()
        .nest(example_route::PATH, example_route::mk_router())
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

    let mut req_scoped_state = framework::ReqScopedState::new(req_id, None);

    if let Some(session_id) = jar.get("session-id").map(|c| c.value()) {
        req_scoped_state.session = framework::find_session(session_id).await;
    }
    req.extensions_mut().insert(req_scoped_state);

    Ok(next.run(req).await)
}

async fn log(
    mut req: extract::Request,
    next: middleware::Next,
) -> Result<response::Response, StatusCode> {
    if let Some(item) = req.extensions().get::<ReqScopedState>() {
        println!("hi, {}", item.ts);
        let r = next.run(req).await;
        println!("bye, {}", Utc::now());
        return Ok(r);
    }

    panic!("not-setup")
}
