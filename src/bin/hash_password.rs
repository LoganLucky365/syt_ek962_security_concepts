// Password hash for initial admin
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, Algorithm, Params, Version,
};
use std::io::{self, Write};

fn main() {
    print!("Enter password to hash: ");
    io::stdout().flush().unwrap();

    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();
    let password = password.trim();

    if password.len() < 12 {
        eprintln!("Error: Password must be at least 12 characters");
        std::process::exit(1);
    }

    let params = Params::new(
        64 * 1024,  
        3,        
        4,        
        None,
    )
    .expect("Invalid Argon2 parameters");

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let salt = SaltString::generate(&mut OsRng);

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password");

    println!("\nPassword hash (use this in your config):\n");
    println!("{}", hash);
    println!("\nFor initial_admin.json:");
    println!(r#"  "password_hash": "{}""#, hash);
    println!("\nFor environment variable:");
    println!("  INITIAL_ADMIN_PASSWORD_HASH='{}'", hash);
}
