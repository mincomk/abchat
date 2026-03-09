use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub sub: String, // username
    pub exp: usize,
    pub is_admin: bool,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a JWT token
    Token {
        /// Username to sign in the token
        #[arg(short, long)]
        username: String,

        /// Secret key for signing (must match backend config)
        #[arg(short, long)]
        secret: String,

        /// Expiration in days
        #[arg(short, long, default_value_t = 7)]
        days: i64,

        /// Set is_admin claim
        #[arg(short, long)]
        admin: bool,
    },
    /// Hash a password using Argon2
    Hash {
        /// Password to hash
        #[arg(short, long)]
        password: String,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Token {
            username,
            secret,
            days,
            admin,
        } => {
            let expiration = Utc::now()
                .checked_add_signed(Duration::days(days))
                .expect("invalid timestamp")
                .timestamp() as usize;

            let claims = Claims {
                sub: username.clone(),
                exp: expiration,
                is_admin: admin,
            };

            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(secret.as_bytes()),
            )
            .expect("Failed to encode token");

            println!("JWT Token for user '{}':", username);
            println!("{}", token);
        }
        Commands::Hash { password } => {
            let hash = persistence::InMemoryPersistence::hash_password(&password)
                .expect("Failed to hash password");
            println!("Argon2 hash for password:");
            println!("{}", hash);
        }
    }
}
