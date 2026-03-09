use clap::Parser;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // username
    pub exp: usize,
    pub is_admin: bool,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "JWT Signer for dbridge web backend", long_about = None)]
struct Args {
    /// JWT secret key
    #[arg(short, long, env = "JWT_SECRET")]
    secret: String,

    /// Username (sub claim)
    #[arg(short, long)]
    username: String,

    /// Expiration time in hours (default: 24)
    #[arg(short, long, default_value_t = 24)]
    expires: u64,

    #[arg(long)]
    admin: bool,
}

fn main() {
    let args = Args::parse();

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let expiration = since_the_epoch.as_secs() + (args.expires * 3600);

    let claims = Claims {
        sub: args.username,
        exp: expiration as usize,
        is_admin: args.admin,
    };

    let token = match encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(args.secret.as_bytes()),
    ) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error signing JWT: {}", e);
            std::process::exit(1);
        }
    };

    println!("{}", token);
}
