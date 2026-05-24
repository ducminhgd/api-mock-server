//! Creates an admin user.
//!
//! Usage:
//!   DATABASE_URL=sqlite://./dev.db create-admin <username>
//!
//! The password is read interactively from the terminal (not echoed).

use std::env;
use std::process;
use std::sync::Arc;

use api_mock_server::application::dto::user::CreateUserRequest;
use api_mock_server::application::services::users::UserService;
use api_mock_server::domain::user::UserRole;
use api_mock_server::infrastructure::auth::password::BcryptHasher;
use api_mock_server::infrastructure::db;
use api_mock_server::infrastructure::db::users::SqlxUserRepository;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: create-admin <username>");
        process::exit(1);
    }
    let username = args[1].clone();

    let password = rpassword::prompt_password(format!("Password for '{username}': "))
        .unwrap_or_else(|e| {
            eprintln!("Failed to read password: {e}");
            process::exit(1);
        });

    if password.is_empty() {
        eprintln!("Password must not be empty.");
        process::exit(1);
    }

    let confirm = rpassword::prompt_password("Confirm password: ").unwrap_or_else(|e| {
        eprintln!("Failed to read password: {e}");
        process::exit(1);
    });

    if password != confirm {
        eprintln!("Passwords do not match.");
        process::exit(1);
    }

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("DATABASE_URL is not set");
        process::exit(1);
    });

    let pool = db::connect(&database_url).await.unwrap_or_else(|e| {
        eprintln!("Failed to connect to database: {e}");
        process::exit(1);
    });

    db::migrate(&pool).await.unwrap_or_else(|e| {
        eprintln!("Migration failed: {e}");
        process::exit(1);
    });

    let service = UserService::new(
        Arc::new(SqlxUserRepository::new(pool)),
        Arc::new(BcryptHasher::default()),
    );

    match service
        .create(CreateUserRequest {
            username: username.clone(),
            password,
            group_id: None,
            role: Some(UserRole::Admin),
        })
        .await
    {
        Ok(_) => println!("Admin user '{username}' created."),
        Err(e) => {
            eprintln!("Failed to create admin user: {e}");
            process::exit(1);
        }
    }
}
