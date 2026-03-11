use super::Persistence;
use crate::{
    AppError, AppResult, Message, MessageUser, NotificationMode, ServiceError, Subscription, User,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};

pub struct PostgresPersistence {
    pool: PgPool,
}

impl PostgresPersistence {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect(url: &str) -> AppResult<Self> {
        let pool = PgPool::connect(url)
            .await
            .map_err(Into::into)
            .map_err(AppError::Service)?;

        Ok(Self::new(pool))
    }

    pub async fn init_db(&self) -> AppResult<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                username TEXT PRIMARY KEY,
                nickname TEXT NOT NULL,
                is_admin BOOLEAN NOT NULL,
                password_hash TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                timestamp BIGINT NOT NULL,
                channel_id TEXT NOT NULL,
                sender_username TEXT NOT NULL,
                sender_nickname TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_messages_channel_id ON messages (channel_id)")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS subscriptions (
                username TEXT NOT NULL,
                endpoint TEXT PRIMARY KEY,
                p256dh TEXT NOT NULL,
                auth TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS user_settings (
                username TEXT PRIMARY KEY,
                notification_mode TEXT NOT NULL DEFAULT 'All'
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        Ok(())
    }
}

#[async_trait]
impl Persistence for PostgresPersistence {
    async fn save_user(&self, u: User) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO users (username, nickname, is_admin, password_hash)
            VALUES ($1, $2, $3, '')
            ON CONFLICT (username) DO UPDATE SET
            nickname = EXCLUDED.nickname,
            is_admin = EXCLUDED.is_admin",
        )
        .bind(&u.username)
        .bind(&u.nickname)
        .bind(u.is_admin)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        Ok(())
    }

    async fn list_users(&self, limit: u32, offset: u32) -> AppResult<Vec<User>> {
        let rows = sqlx::query("SELECT username, nickname, is_admin FROM users LIMIT $1 OFFSET $2")
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        let users = rows
            .into_iter()
            .map(|r| User {
                username: r.get("username"),
                nickname: r.get("nickname"),
                is_admin: r.get("is_admin"),
            })
            .collect();

        Ok(users)
    }

    async fn get_user(&self, username: &str) -> AppResult<Option<User>> {
        let row = sqlx::query("SELECT username, nickname, is_admin FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        let user = row.map(|r| User {
            username: r.get("username"),
            nickname: r.get("nickname"),
            is_admin: r.get("is_admin"),
        });

        Ok(user)
    }

    async fn delete_user(&self, username: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(username)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        Ok(())
    }

    async fn get_password_hash(&self, username: &str) -> AppResult<Option<String>> {
        let row = sqlx::query("SELECT password_hash FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        Ok(row.map(|r| r.get("password_hash")))
    }

    async fn set_password_hash(&self, username: &str, hash: &str) -> AppResult<()> {
        sqlx::query("UPDATE users SET password_hash = $1 WHERE username = $2")
            .bind(hash)
            .bind(username)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        Ok(())
    }

    async fn add_message(&self, message: Message) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO messages (id, content, timestamp, channel_id, sender_username, sender_nickname)
            VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&message.id)
        .bind(&message.content)
        .bind(message.timestamp as i64)
        .bind(&message.channel_id)
        .bind(&message.sender.username)
        .bind(&message.sender.nickname)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        Ok(())
    }

    async fn list_messages(
        &self,
        channel_id: String,
        limit: u32,
        offset: u32,
    ) -> AppResult<Vec<Message>> {
        let rows = sqlx::query(
            "SELECT 
                id, content, timestamp, channel_id,
                sender_username, sender_nickname
            FROM messages
            WHERE channel_id = $1 
            ORDER BY timestamp DESC 
            LIMIT $2 OFFSET $3",
        )
        .bind(channel_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        let messages = rows
            .into_iter()
            .map(|r| Message {
                id: r.get("id"),
                content: r.get("content"),
                timestamp: r.get::<i64, _>("timestamp") as u64,
                channel_id: r.get("channel_id"),
                sender: MessageUser {
                    username: r.get("sender_username"),
                    nickname: r.get("sender_nickname"),
                },
            })
            .collect();

        Ok(messages)
    }

    async fn add_subscription(&self, sub: Subscription) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO subscriptions (username, endpoint, p256dh, auth)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (endpoint) DO UPDATE SET
            username = EXCLUDED.username,
            p256dh = EXCLUDED.p256dh,
            auth = EXCLUDED.auth",
        )
        .bind(&sub.username)
        .bind(&sub.endpoint)
        .bind(&sub.p256dh)
        .bind(&sub.auth)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        Ok(())
    }

    async fn list_subscriptions(&self, username: &str) -> AppResult<Vec<Subscription>> {
        let rows = sqlx::query(
            "SELECT username, endpoint, p256dh, auth FROM subscriptions WHERE username = $1",
        )
        .bind(username)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        let subs = rows
            .into_iter()
            .map(|r| Subscription {
                username: r.get("username"),
                endpoint: r.get("endpoint"),
                p256dh: r.get("p256dh"),
                auth: r.get("auth"),
            })
            .collect();

        Ok(subs)
    }

    async fn delete_subscription(&self, endpoint: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM subscriptions WHERE endpoint = $1")
            .bind(endpoint)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        Ok(())
    }

    async fn delete_user_subscriptions(&self, username: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM subscriptions WHERE username = $1")
            .bind(username)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        Ok(())
    }

    async fn get_user_notification_mode(&self, username: &str) -> AppResult<NotificationMode> {
        let row = sqlx::query("SELECT notification_mode FROM user_settings WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        Ok(row
            .map(|r| r.get::<String, _>("notification_mode").into())
            .unwrap_or(NotificationMode::All))
    }

    async fn set_user_notification_mode(
        &self,
        username: &str,
        mode: NotificationMode,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO user_settings (username, notification_mode)
            VALUES ($1, $2)
            ON CONFLICT (username) DO UPDATE SET
            notification_mode = EXCLUDED.notification_mode",
        )
        .bind(username)
        .bind(mode.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        Ok(())
    }

    async fn get_subscriptions_by_mode(
        &self,
        modes: Vec<NotificationMode>,
    ) -> AppResult<Vec<Subscription>> {
        let modes_str: Vec<String> = modes.into_iter().map(|m| m.to_string()).collect();

        let rows = sqlx::query(
            "SELECT s.username, s.endpoint, s.p256dh, s.auth 
             FROM subscriptions s
             LEFT JOIN user_settings us ON s.username = us.username
             WHERE COALESCE(us.notification_mode, 'All') = ANY($1)",
        )
        .bind(&modes_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        let subs = rows
            .into_iter()
            .map(|r| Subscription {
                username: r.get("username"),
                endpoint: r.get("endpoint"),
                p256dh: r.get("p256dh"),
                auth: r.get("auth"),
            })
            .collect();

        Ok(subs)
    }
}
