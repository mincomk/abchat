use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum AuthError {
    #[error("Authorization header required")]
    HeaderRequired,

    #[error("Account does not exist")]
    AccountNotExist,

    #[error("You don't have an access to this route")]
    NoAccess,

    #[error("Invalid token provided")]
    InvalidToken,

    #[error("Already authenticated")]
    AlreadyAuthenticated,

    #[error("Unauthorized")]
    Unauthorized,
}
