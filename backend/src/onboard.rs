use crate::{AppConfig, AppResult, AppState, User};

/// Checks if admin credentials is present in config and creates if so.
pub async fn init_admin_account(config: &AppConfig, state: &AppState) -> AppResult<()> {
    if let (Some(username), Some(password_hash)) =
        (&config.admin_username, &config.admin_password_hash)
    {
        tracing::info!("Admin account is present. Updating account DB.");

        state
            .persistence
            .save_user(User {
                username: username.to_string(),
                nickname: username.to_string(),
                is_admin: true,
            })
            .await?;
        
        state
            .persistence
            .set_password_hash(username, password_hash)
            .await?;
    } else {
        tracing::info!("Admin account is not present. Skipping admin creation.");
    }

    Ok(())
}
