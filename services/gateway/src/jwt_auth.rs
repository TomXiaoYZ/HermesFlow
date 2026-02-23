use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use base64::{engine::general_purpose, Engine as _};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
}

/// Load the JWT secret from env. The Java JwtService stores it as a hex string
/// that is Base64-decoded to get the raw HMAC key bytes:
///   `Decoders.BASE64.decode(secretKey)` → raw bytes → `Keys.hmacShaKeyFor()`
/// We replicate the same: Base64-decode the env var to get the raw HMAC key.
fn get_decoding_key() -> Option<DecodingKey> {
    let secret = std::env::var("JWT_SECRET").ok().filter(|s| !s.is_empty())?;
    let key_bytes = general_purpose::STANDARD
        .decode(secret.as_bytes())
        .or_else(|_| {
            // The Java default is a hex string that happens to also be valid
            // when decoded as raw UTF-8 bytes (used as HMAC key material).
            // Fall back to treating it as raw bytes if Base64 fails.
            Ok::<Vec<u8>, ()>(secret.into_bytes())
        })
        .ok()?;
    Some(DecodingKey::from_secret(&key_bytes))
}

/// Check if JWT enforcement is enabled (JWT_SECRET must be set).
/// When disabled, all requests pass through without auth — allows gradual
/// frontend integration of login flow before enforcing JWT globally.
pub fn is_jwt_enabled() -> bool {
    std::env::var("JWT_SECRET")
        .map(|s| !s.is_empty())
        .unwrap_or(false)
}

/// Axum middleware: validates JWT from `Authorization: Bearer <token>` header.
/// On success, injects `Claims` into request extensions.
/// When JWT_SECRET is not set, requests pass through unauthenticated.
pub async fn jwt_middleware(mut req: Request<Body>, next: Next) -> Response {
    let key = match get_decoding_key() {
        Some(k) => k,
        None => {
            // JWT not configured — pass through (allows frontend to work
            // before login UI is integrated). Log once at startup via is_jwt_enabled().
            return next.run(req).await;
        }
    };

    let token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let token = match token {
        Some(t) => t,
        None => {
            // No token provided but JWT is enabled — reject
            return unauthorized("Missing or invalid Authorization header");
        }
    };

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    match decode::<Claims>(token, &key, &validation) {
        Ok(data) => {
            req.extensions_mut().insert(data.claims);
            next.run(req).await
        }
        Err(e) => {
            warn!("JWT validation failed: {}", e);
            unauthorized("Invalid or expired token")
        }
    }
}

/// Validate a token from WebSocket query parameter.
/// Returns `Ok(Claims)` on success, `Err(message)` on failure.
/// When JWT is not configured, returns a default anonymous claim.
pub fn validate_ws_token(token: Option<&str>) -> Result<Claims, &'static str> {
    let key = match get_decoding_key() {
        Some(k) => k,
        None => {
            // JWT not configured — allow anonymous WebSocket connections
            return Ok(Claims {
                sub: "anonymous".to_string(),
                exp: u64::MAX,
                iat: 0,
            });
        }
    };
    let token = token.ok_or("Missing authentication token")?;

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    decode::<Claims>(token, &key, &validation)
        .map(|data| data.claims)
        .map_err(|_| "Invalid or expired token")
}

fn unauthorized(message: &str) -> Response {
    (
        axum::http::StatusCode::UNAUTHORIZED,
        axum::Json(serde_json::json!({ "error": message })),
    )
        .into_response()
}
