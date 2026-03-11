use backend::auth::hash::hash_password;
use rpassword::read_password;
use std::io::{self, Write};

fn main() {
    print!("Enter your password: ");
    io::stdout().flush().unwrap();

    let password = read_password().unwrap();

    let hash = hash_password(&password).unwrap();

    println!("Hash:\n{}", hash);
}
