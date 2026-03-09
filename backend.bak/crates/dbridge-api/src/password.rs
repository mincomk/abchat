use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use dbridge_error::DBridgeError;

pub fn hash_password(password: &str) -> Result<String, DBridgeError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| DBridgeError::Internal(e.to_string()))?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let argon2 = Argon2::default();
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
