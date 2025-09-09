use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use async_trait::async_trait;
use crate::utils::jwt::{decode_token, Claims};
use jsonwebtoken::errors::{Error as JwtError, ErrorKind};
use tokio_tungstenite::tungstenite::Message;

pub struct WsAuth(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for WsAuth
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let query = parts.uri.query().unwrap_or("");
        let token = query
            .split('&')
            .find_map(|kv| {
                let mut split = kv.split('=');
                let key = split.next()?;
                let val = split.next()?;
                if key == "token" {
                    Some(val.to_string())
                } else {
                    None
                }
            });

        let token = match token {
            Some(t) => t,
            None => return Err((StatusCode::BAD_REQUEST, "Missing token in query".into())),
        };

        Self::validate_token(&token)
    }
}

impl WsAuth {
    pub fn validate_token(token: &str) -> Result<Self, (StatusCode, String)> {
        match decode_token(token) {
            Ok(claims) => Ok(WsAuth(claims)),
            Err(e) => {
                if let Some(jwt_err) = e.downcast_ref::<JwtError>() {
                    match jwt_err.kind() {
                        ErrorKind::ExpiredSignature => {
                            Err((StatusCode::UNAUTHORIZED, "Token expired".into()))
                        }
                        ErrorKind::InvalidToken => {
                            Err((StatusCode::UNAUTHORIZED, "Invalid token".into()))
                        }
                        ErrorKind::InvalidSignature => {
                            Err((StatusCode::UNAUTHORIZED, "Invalid signature".into()))
                        }
                        _ => Err((StatusCode::UNAUTHORIZED, "Invalid token".into())),
                    }
                } else {
                    Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal auth error".into()))
                }
            }
        }
    }

    pub async fn from_first_message(msg: &Message) -> Result<Self, (StatusCode, String)> {
        match msg {
            Message::Text(text) => {
                let json = serde_json::from_str::<serde_json::Value>(text)
                    .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid JSON".to_string()))?;
                let token = json.get("token")
                    .and_then(|v| v.as_str())
                    .ok_or((StatusCode::BAD_REQUEST, "Missing token field".into()))?;

                Self::validate_token(token)
            }
            _ => Err((StatusCode::BAD_REQUEST, "Expected text message".into())),
        }
    }
}