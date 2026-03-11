use crate::AppError;
use crate::AppResult;
use crate::error::ServiceError;
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            tracing::error!("Password hashing error: {:?}", e);
            AppError::Service(ServiceError::Internal("Password hashing failed".into()))
        })?
        .to_string();

    Ok(password_hash)
}

pub fn verify_password(hash: &str, password: &str) -> AppResult<bool> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| {
        tracing::error!("Invalid password hash: {:?}", e);
        AppError::Service(ServiceError::Internal(
            "Invalid password hash stored".into(),
        ))
    })?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
