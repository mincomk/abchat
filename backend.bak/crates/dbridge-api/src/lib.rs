use axum::{
    Router,
    extract::{FromRef, FromRequestParts, Path, State},
    http::{StatusCode, request::Parts},
    response::IntoResponse,
    routing::{delete, get, post},
};
use dbridge_core::User;
use dbridge_error::DBridgeError;
use dbridge_persistence::{Account, Persistence};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::password::{hash_password, verify_password};

mod password;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // username
    pub exp: usize,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: Option<String>,
    pub nickname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountResponse {
    pub username: String,
    pub nickname: String,
    pub is_admin: bool,
}

#[derive(Clone)]
pub struct ApiState {
    pub persistence: Arc<dyn Persistence>,
    pub jwt_secret: String,
    pub admin_users: Vec<String>,
}

pub fn create_router<S>() -> Router<S>
where
    S: Send + Sync + 'static + Clone,
    Arc<ApiState>: FromRef<S>,
{
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/auth/login", post(login_handler))
        .route("/admin/register", post(register_handler))
        .route("/admin/accounts", get(list_accounts_handler))
        .route("/admin/accounts/{username}", delete(delete_account_handler))
        .layer(cors)
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync + Clone,
    Arc<ApiState>: FromRef<S>,
{
    type Rejection = DBridgeError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let api_state = Arc::<ApiState>::from_ref(state);
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| DBridgeError::Auth("Missing Authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(DBridgeError::Auth(
                "Invalid Authorization header format".to_string(),
            ));
        }

        let token = &auth_header[7..];
        let decoding_key = DecodingKey::from_secret(api_state.jwt_secret.as_bytes());
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|_| DBridgeError::Auth("Invalid token".to_string()))?;

        // Verify account exists in persistence
        let account: Option<Account> = api_state
            .persistence
            .get_account(&token_data.claims.sub)
            .await?;

        let account =
            account.ok_or_else(|| DBridgeError::Auth("Account no longer exists".to_string()))?;

        if account.is_admin != token_data.claims.is_admin {
            return Err(DBridgeError::Auth("Token claims mismatch".to_string()));
        }

        Ok(token_data.claims)
    }
}

async fn login_handler(
    State(state): State<Arc<ApiState>>,
    axum::Json(payload): axum::Json<AuthPayload>,
) -> Result<impl IntoResponse, DBridgeError> {
    let account: Option<Account> = state.persistence.get_account(&payload.username).await?;
    let account = account.ok_or_else(|| DBridgeError::Auth("Invalid credentials".to_string()))?;

    let password = payload
        .password
        .ok_or_else(|| DBridgeError::Validation("Password required".to_string()))?;

    if !verify_password(&password, &account.password_hash) {
        return Err(DBridgeError::Auth("Invalid credentials".to_string()));
    }

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("invalid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: account.username.clone(),
        exp: expiration,
        is_admin: account.is_admin,
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|e| DBridgeError::Internal(e.to_string()))?;

    let user = User {
        username: account.username,
        nickname: account.nickname,
        is_admin: account.is_admin,
    };

    Ok((StatusCode::OK, axum::Json(AuthResponse { token, user })))
}

async fn register_handler(
    State(state): State<Arc<ApiState>>,
    claims: Claims,
    axum::Json(payload): axum::Json<AuthPayload>,
) -> Result<impl IntoResponse, DBridgeError> {
    if !claims.is_admin {
        return Err(DBridgeError::Auth(
            "Only admins can register users".to_string(),
        ));
    }

    let password = payload
        .password
        .ok_or_else(|| DBridgeError::Validation("Password required".to_string()))?;

    let password_hash = hash_password(&password)?;

    // Check if account already exists
    let existing: Option<Account> = state.persistence.get_account(&payload.username).await?;
    if existing.is_some() {
        return Err(DBridgeError::Validation(
            "Account already exists".to_string(),
        ));
    }

    let is_admin = state.admin_users.contains(&payload.username);

    let account = Account {
        username: payload.username.clone(),
        password_hash,
        nickname: payload.nickname.unwrap_or(payload.username.clone()),
        is_admin,
    };

    state.persistence.save_account(account).await?;

    Ok(StatusCode::CREATED)
}

async fn list_accounts_handler(
    State(state): State<Arc<ApiState>>,
    claims: Claims,
) -> Result<impl IntoResponse, DBridgeError> {
    if !claims.is_admin {
        return Err(DBridgeError::Auth(
            "Only admins can list accounts".to_string(),
        ));
    }

    let accounts = state.persistence.list_accounts().await?;
    let sanitized: Vec<AccountResponse> = accounts
        .into_iter()
        .map(|a| AccountResponse {
            username: a.username,
            nickname: a.nickname,
            is_admin: a.is_admin,
        })
        .collect();

    Ok((StatusCode::OK, axum::Json(sanitized)))
}

async fn delete_account_handler(
    State(state): State<Arc<ApiState>>,
    claims: Claims,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, DBridgeError> {
    if !claims.is_admin {
        return Err(DBridgeError::Auth(
            "Only admins can delete accounts".to_string(),
        ));
    }

    if claims.sub == username {
        return Err(DBridgeError::Validation(
            "Cannot delete yourself".to_string(),
        ));
    }

    state.persistence.delete_account(&username).await?;
    Ok(StatusCode::NO_CONTENT)
}
