use crate::{db::openid_connect_states, framework::AppState, settings::OPENID_CONNECT_STATE_KEY};
use axum::{
    extract,
    http::StatusCode,
    response::{self, IntoResponse, Redirect},
};
use axum::{routing, Router};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use sea_orm::{ActiveModelTrait, Set};

/// パス
pub const PATH: &'static str = "/openid-connect";

pub fn mk_router() -> Router<AppState> {
    Router::new()
        .route("/", routing::get(handler))
        .route("/callback", routing::get(callback_handler))
}

pub async fn handler(
    extract::State(state): extract::State<AppState>,

    jar: CookieJar,
) -> impl IntoResponse {
    let csrf_token = ulid::Ulid::new().to_string();
    let sid = ulid::Ulid::new().to_string();

    let t = openid_connect_states::ActiveModel {
        sid: Set(sid.clone()),
        state: Set(csrf_token),
    };

    if let Ok(model) = t.insert(&state.db_client).await {
        let jar = jar.add(Cookie::new(OPENID_CONNECT_STATE_KEY, sid));

        (jar, Redirect::to("http://localhost:3000/login")).into_response()
    } else {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub async fn callback_handler(
    extract::State(state): extract::State<AppState>,
) -> impl IntoResponse {
    StatusCode::NO_CONTENT
}
