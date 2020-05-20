use crate::*;
use sha2::{Digest, Sha512};
use std::env;
use std::fmt;
use tokio::sync::Mutex;

fn hash_password(password: &str) -> Vec<u8> {
    let mut hasher = Sha512::new();
    hasher.input(password);
    hasher.input(env::var("ROCKET_SECRET_KEY").unwrap_or("<<dev>>".into()));
    let hash = hasher.result();
    hash.to_vec()
}

impl SharedContext {
    pub async fn find_user(&self, id: Uuid) -> Option<User> {
        let conn = self.db.get().await.ok()?;

        let row = conn
            .query_one("SELECT username, email FROM users WHERE id = $1", &[&id])
            .await
            .ok()?;

        let username = row.get::<usize, String>(0);
        let email = row.get::<usize, String>(1);

        Some(User {
            id,
            username,
            email,
        })
    }

    pub async fn sign_up(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<User, SignUpError> {
        let id = Uuid::new_v4();
        let password = hash_password(password);

        let conn = self.db.get().await.unwrap();

        let row = conn
            .query_one(
                "SELECT username, email FROM sign_up($1, $2, $3, $4)",
                &[&id, &username, &email, &password],
            )
            .await?;

        let username = row.get::<usize, String>(0);
        let email = row.get::<usize, String>(1);

        Ok(User {
            id,
            username,
            email,
        })
    }

    pub async fn sign_in(
        &self,
        username_or_email: &str,
        password: &str,
    ) -> Result<User, SignInError> {
        let password = hash_password(password);

        let conn = self.db.get().await.unwrap();

        let row = conn
            .query_one(
                "SELECT id, username, email FROM sign_in($1, $2)",
                &[&username_or_email, &password],
            )
            .await?;

        let id = row
            .try_get::<usize, Uuid>(0)
            .map_err(|_| SignInError::InvalidCredentials)?;
        let username = row.get::<usize, String>(1);
        let email = row.get::<usize, String>(2);

        Ok(User {
            id,
            username,
            email,
        })
    }
}

#[derive(Debug)]
pub enum SignUpError {
    Unknown,
    DuplicateEmail,
    DuplicateUsername,
}

impl fmt::Display for SignUpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SignUpError::Unknown => write!(f, "An unknown error occured"),
            SignUpError::DuplicateEmail => {
                write!(f, "A user is already registered with this email address")
            }
            SignUpError::DuplicateUsername => write!(f, "This username is taken"),
        }
    }
}

impl From<tokio_postgres::Error> for SignUpError {
    fn from(e: tokio_postgres::Error) -> Self {
        let msg = format!("{}", e);
        if msg.contains("users_username_key") {
            SignUpError::DuplicateUsername
        } else if msg.contains("users_email_key") {
            SignUpError::DuplicateEmail
        } else {
            SignUpError::Unknown
        }
    }
}

#[derive(Debug)]
pub enum SignInError {
    Unknown,
    InvalidCredentials,
}

impl fmt::Display for SignInError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SignInError::Unknown => write!(f, "An unknown error occured"),
            SignInError::InvalidCredentials => write!(f, "Invalid credentials"),
        }
    }
}

impl From<tokio_postgres::Error> for SignInError {
    fn from(_e: tokio_postgres::Error) -> Self {
        SignInError::Unknown
    }
}

impl Context {
    pub async fn did_sign_in(&self) -> bool {
        self.did_sign_in.lock().await.clone()
    }

    pub async fn did_sign_out(&self) -> bool {
        self.did_sign_out.lock().await.clone()
    }

    pub fn user(&self) -> &Mutex<Option<User>> {
        &self.user
    }

    async fn record_did_sign_in(&self, user: User) {
        let mut u = self.user.lock().await;
        *u = Some(user);

        let mut did_sign_in = self.did_sign_in.lock().await;
        *did_sign_in = true;

        let mut did_sign_out = self.did_sign_out.lock().await;
        *did_sign_out = false;
    }

    async fn record_did_sign_out(&self) {
        let mut u = self.user.lock().await;
        *u = None;

        let mut did_sign_in = self.did_sign_in.lock().await;
        *did_sign_in = false;

        let mut did_sign_out = self.did_sign_out.lock().await;
        *did_sign_out = true;
    }

    pub async fn sign_up(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<User, SignUpError> {
        let user = self.shared.sign_up(username, email, password).await?;
        self.record_did_sign_in(user.clone()).await;
        Ok(user)
    }

    pub async fn sign_in(
        &self,
        username_or_email: &str,
        password: &str,
    ) -> Result<User, SignInError> {
        let user = self.shared.sign_in(username_or_email, password).await?;
        self.record_did_sign_in(user.clone()).await;
        Ok(user)
    }

    pub async fn sign_out(&self) {
        self.record_did_sign_out().await;
    }
}
