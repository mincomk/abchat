use super::Persistence;
use crate::{AppError, AppResult, Message, ServiceError, User};
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
                sender_username TEXT NOT NULL REFERENCES users(username)
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        sqlx::query(
            "ALTER TABLE messages ADD COLUMN IF NOT EXISTS sender_username TEXT REFERENCES users(username)",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Service(ServiceError::Database(e)))?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_messages_channel_id ON messages (channel_id)",
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

    async fn list_users(&self) -> AppResult<Vec<User>> {
        let rows = sqlx::query("SELECT username, nickname, is_admin FROM users")
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
            "INSERT INTO messages (id, content, timestamp, channel_id, sender_username)
            VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&message.id)
        .bind(&message.content)
        .bind(message.timestamp as i64)
        .bind(&message.channel_id)
        .bind(&message.sender.username)
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
                m.id, m.content, m.timestamp, m.channel_id,
                u.username, u.nickname, u.is_admin
            FROM messages m
            JOIN users u ON m.sender_username = u.username
            WHERE m.channel_id = $1 
            ORDER BY m.timestamp DESC 
            LIMIT $2 OFFSET $3"
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
                sender: User {
                    username: r.get("username"),
                    nickname: r.get("nickname"),
                    is_admin: r.get("is_admin"),
                },
            })
            .collect();

        Ok(messages)
    }
}
