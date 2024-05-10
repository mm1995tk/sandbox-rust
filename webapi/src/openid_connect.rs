use crate::{
    framework::{
        logger::Logger,
        session::mk_cookie,
        system::{AppError, IntoAppError, Panic},
        AppState, ReqScopedState,
    },
    settings::OPENID_CONNECT_STATE_KEY,
};
use axum::{
    extract::{self, Query},
    response::{IntoResponse, Redirect, Response},
};
use axum::{routing, Router};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use jsonwebtoken::{decode, jwk::JwkSet, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use ulid::Ulid;

/// パス
pub const PATH: &str = "/openid-connect";

pub fn mk_router() -> Router<AppState> {
    Router::new()
        .route("/", routing::get(handler))
        .route("/callback", routing::get(callback_handler))
}

async fn handler(
    extract::State(state): extract::State<AppState>,
    ctx: ReqScopedState,
    jar: CookieJar,
    logger: Logger,
) -> Result<Response, AppError> {
    let authorization_endpoint = state
        .discovery_json
        .get("authorization_endpoint")
        .and_then(|v| v.as_str())
        .ok_or(
            Panic::new("authorization_endpointが見つからない".to_string())
                .into_app_error(logger.clone(), &ctx.req_id),
        )?;

    let csrf_token = ulid::Ulid::new().to_string();
    let nonce = ulid::Ulid::new().to_string();

    let query_params: Vec<(&str, &str)> = vec![
        ("state", &csrf_token),
        ("client_id", &state.env.google_client_id),
        ("response_type", "code"),
        ("scope", "openid profile email"),
        ("redirect_uri", &state.env.google_redirect_uri),
        ("nonce", &nonce),
        ("access_type", "offline"),
    ];

    let client_redirct_url = reqwest::Url::parse_with_params(authorization_endpoint, &query_params)
        .map_err(|e| Panic::new(e).into_app_error(logger.clone(), &ctx.req_id))?;

    let redirect = Redirect::to(client_redirct_url.as_str());

    let jar = jar.add({
        let mut cookie = Cookie::new(OPENID_CONNECT_STATE_KEY, csrf_token);
        cookie.set_secure(true);
        cookie.set_http_only(true);
        cookie
    });
    Ok((jar, redirect).into_response())
}

#[derive(Deserialize)]
struct Params {
    code: String,
    state: String,
}
async fn callback_handler(
    Query(params): Query<Params>,
    extract::State(app_state): extract::State<AppState>,
    jar: CookieJar,
    ctx: ReqScopedState,
    logger: Logger,
) -> Result<Response, AppError> {
    validate_state_hash(&params.state, &jar)?;

    let tokens = get_tokens(&params.code, &app_state, &ctx, &logger).await?;
    let valid_id_token = extract_id_token(&tokens, &app_state, &ctx, &logger).await?;

    println!(
        "login: {}; email: {}",
        &valid_id_token.name, &valid_id_token.email
    );

    let response = (
        add_session_id(remove_state_hash(jar)),
        Redirect::to("/login"),
    );

    Ok(response.into_response())
}

fn validate_state_hash(state: &str, jar: &CookieJar) -> Result<(), AppError> {
    let state_in_cookie =
        jar.get(OPENID_CONNECT_STATE_KEY)
            .map(|c| c.value())
            .ok_or(AppError::AutorizationError(
                "state値がcookieに含まれていない".to_string(),
            ))?;

    if state != state_in_cookie {
        return Err(AppError::AutorizationError("不正なstate値".to_string()));
    }

    Ok(())
}

async fn get_tokens(
    code: &str,
    app_state: &AppState,
    ctx: &ReqScopedState,
    logger: &Logger,
) -> Result<Value, AppError> {
    let token_endpoint = app_state
        .discovery_json
        .get("token_endpoint")
        .and_then(|v| v.as_str())
        .ok_or(
            Panic::new("token_endpointが見つからない".to_string())
                .into_app_error(logger.clone(), &ctx.req_id),
        )?;

    let body = json!({
        "code": code,
        "client_id": &app_state.env.google_client_id,
        "client_secret": &app_state.env.google_client_secret,
        "redirect_uri": &app_state.env.google_redirect_uri,
        "grant_type": "authorization_code"
    });

    let error_response = |e| Panic::new(e).into_app_error(logger.clone(), &ctx.req_id);

    reqwest::Client::new()
        .post(token_endpoint)
        .body(body.to_string())
        .send()
        .await
        .map_err(error_response)?
        .json::<Value>()
        .await
        .map_err(error_response)
}

async fn extract_id_token(
    tokens: &Value,
    app_state: &AppState,
    ctx: &ReqScopedState,
    logger: &Logger,
) -> Result<Claims, AppError> {
    let error_response =
        |e: &str| Panic::new(e.to_string()).into_app_error(logger.clone(), &ctx.req_id);

    let id_token = tokens
        .get("id_token")
        .and_then(|v| v.as_str())
        .ok_or(error_response("token_endpointが見つからない"))?;

    let jwks_uri = app_state
        .discovery_json
        .get("jwks_uri")
        .and_then(|v| v.as_str())
        .ok_or(error_response("jwks_uriが見つからない"))?;

    let error_response = |e| Panic::new(e).into_app_error(logger.clone(), &ctx.req_id);

    let jwk_set = reqwest::get(jwks_uri)
        .await
        .map_err(error_response)?
        .json::<JwkSet>()
        .await
        .map_err(error_response)?;

    let validation = {
        let mut tmp = Validation::new(jsonwebtoken::Algorithm::RS256);
        tmp.set_audience(&[&app_state.env.google_client_id]);
        tmp
    };

    let decode_claims = |jwk| {
        decode::<Claims>(
            id_token,
            &DecodingKey::from_jwk(jwk).expect("base64decodeできること"),
            &validation,
        )
    };

    jwk_set
        .keys
        .iter()
        .map(decode_claims)
        .find_map(|item| item.ok())
        .ok_or(AppError::AuthenticationError)
        .map(|item| item.claims)
}

fn add_session_id(jar: CookieJar) -> CookieJar {
    jar.add(mk_cookie(Ulid::new().to_string()))
}

fn remove_state_hash(jar: CookieJar) -> CookieJar {
    jar.remove({
        let mut c = Cookie::from(OPENID_CONNECT_STATE_KEY);
        c.set_path("/");
        c
    })
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    name: String,
    email: String,
    exp: i64,
}
