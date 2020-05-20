use crate::*;
use sha2::{Digest, Sha512};
use std::env;
use std::fmt;
use tokio::sync::Mutex;

fn hash_password(password: &str) -> Vec<u8> {
    let mut hasher = Sha512::new();
    hasher.input(password);
    hasher.input(env::var("PASSWORD_SALT").unwrap_or("<<dev>>".into()));
    let hash = hasher.result();
    hash.to_vec()
}

#[derive(Debug, Clone)]
pub enum UserValidationError {
    UsernameTooShort,
    PasswordTooShort,
    InvalidEmail,
}

const USERNAME_LENGTH_LIMIT: usize = 6;
const PASSWORD_LENGTH_LIMIT: usize = 6;

impl UserValidationError {
    fn validate_username(username: &str) -> Result<(), UserValidationError> {
        if username.len() < USERNAME_LENGTH_LIMIT {
            return Err(UserValidationError::UsernameTooShort);
        }
        Ok(())
    }

    fn validate_email(email: &str) -> Result<(), UserValidationError> {
        if !email.contains("@") {
            return Err(UserValidationError::InvalidEmail);
        }
        Ok(())
    }

    fn validate_password(password: &str) -> Result<(), UserValidationError> {
        if password.len() < PASSWORD_LENGTH_LIMIT {
            return Err(UserValidationError::PasswordTooShort);
        }
        Ok(())
    }
}

impl fmt::Display for UserValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserValidationError::UsernameTooShort => write!(f, "Username must be at least {} characters long", USERNAME_LENGTH_LIMIT),
            UserValidationError::PasswordTooShort => write!(f, "Password must be at least {} characters long", PASSWORD_LENGTH_LIMIT),
            UserValidationError::InvalidEmail => write!(f, "Invalid email address"),
        }
    }
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
        UserValidationError::validate_username(username)?;
        UserValidationError::validate_email(email)?;
        UserValidationError::validate_password(password)?;

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

    pub async fn remove_account(&self, id: Uuid, password: &str) -> Result<(), RemoveAccountError> {
        let password = hash_password(password);

        let conn = self.db.get().await.unwrap();

        let row = conn
            .query_one("SELECT remove_account($1, $2)", &[&id, &password])
            .await?;

        let succeeded = row.get::<usize, Option<bool>>(0).unwrap_or(false);

        if succeeded {
            Ok(())
        } else {
            Err(RemoveAccountError::Failed)
        }
    }

    pub async fn update_account(
        &self,
        id: Uuid,
        new_email: Option<String>,
        new_username: Option<String>,
        password_change: Option<PasswordChange>,
    ) -> Result<User, UpdateAccountError> {
        let (current_password, new_password) = match password_change {
            None => (None, None),
            Some(change) => {
                UserValidationError::validate_password(&change.new)?;
                (
                    Some(hash_password(&change.current)),
                    Some(hash_password(&change.new)),
                )
            },
        };

        if let Some(email) = &new_email {
            UserValidationError::validate_email(email)?;
        }
        if let Some(username) = &new_username {
            UserValidationError::validate_username(username)?;
        }

        let db = self.db.get().await.unwrap();
        let row = db
            .query_one(
                "
                    UPDATE users
                    SET email = COALESCE($3, email),
                        username = COALESCE($4, username),
                        password = COALESCE($5, password)
                    WHERE id = $1 AND password = COALESCE($2, password)
                    RETURNING id, email, username
                ",
                &[
                    &id,
                    &current_password,
                    &new_email,
                    &new_username,
                    &new_password,
                ],
            )
            .await?;

        let id = row.get::<usize, Uuid>(0);
        let email = row.get::<usize, String>(1);
        let username = row.get::<usize, String>(2);

        Ok(User {
            id,
            username,
            email,
        })
    }
}

#[derive(Debug)]
pub enum UpdateAccountError {
    Failed,
    NotSignedIn,
    DuplicateEmail,
    DuplicateUsername,
    WrongCurrentPassword,
    ValidationFailed(UserValidationError),
}

impl fmt::Display for UpdateAccountError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UpdateAccountError::Failed => write!(f, "Account update failed"),
            UpdateAccountError::NotSignedIn => {
                write!(f, "You must be signed in to update your account")
            }
            UpdateAccountError::DuplicateEmail => {
                write!(f, "An account is already using this email address")
            }
            UpdateAccountError::DuplicateUsername => write!(f, "This username is taken"),
            UpdateAccountError::WrongCurrentPassword => {
                write!(f, "The current password provided is incorrect")
            }
            UpdateAccountError::ValidationFailed(v) => write!(f, "{}", v),
        }
    }
}

impl From<UserValidationError> for UpdateAccountError {
    fn from(e: UserValidationError) -> Self {
        UpdateAccountError::ValidationFailed(e)
    }
}

impl From<tokio_postgres::Error> for UpdateAccountError {
    fn from(e: tokio_postgres::Error) -> Self {
        let msg = format!("{:?}", e);
        if msg.contains("users_username_key") {
            UpdateAccountError::DuplicateUsername
        } else if msg.contains("users_email_key") {
            UpdateAccountError::DuplicateEmail
        } else if msg.contains("RowCount") {
            UpdateAccountError::WrongCurrentPassword
        } else {
            eprintln!("{}", msg);
            UpdateAccountError::Failed
        }
    }
}

#[derive(Debug)]
pub enum RemoveAccountError {
    Failed,
    NotSignedIn,
}

impl fmt::Display for RemoveAccountError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RemoveAccountError::Failed => write!(f, "Account removal failed"),
            RemoveAccountError::NotSignedIn => {
                write!(f, "You must be signed in to remove your account")
            }
        }
    }
}

impl From<tokio_postgres::Error> for RemoveAccountError {
    fn from(_e: tokio_postgres::Error) -> Self {
        RemoveAccountError::Failed
    }
}

#[derive(Debug)]
pub enum SignUpError {
    Unknown,
    DuplicateEmail,
    DuplicateUsername,
    ValidationFailed(UserValidationError),
}

impl fmt::Display for SignUpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SignUpError::Unknown => write!(f, "An unknown error occured"),
            SignUpError::DuplicateEmail => {
                write!(f, "A user is already registered with this email address")
            }
            SignUpError::DuplicateUsername => write!(f, "This username is taken"),
            SignUpError::ValidationFailed(v) => write!(f, "{}", v),
        }
    }
}

impl From<UserValidationError> for SignUpError {
    fn from(e: UserValidationError) -> Self {
        SignUpError::ValidationFailed(e)
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

    pub async fn remove_account(&self, password: &str) -> Result<(), RemoveAccountError> {
        let user = self.user.lock().await;
        match user.as_ref() {
            None => Err(RemoveAccountError::NotSignedIn),
            Some(user) => self.shared.remove_account(user.id, password).await,
        }
    }

    pub async fn update_account(
        &self,
        new_email: Option<String>,
        new_username: Option<String>,
        password_change: Option<PasswordChange>,
    ) -> Result<User, UpdateAccountError> {
        let user = self.user.lock().await;
        match user.as_ref() {
            None => Err(UpdateAccountError::NotSignedIn),
            Some(user) => {
                self.shared
                    .update_account(user.id, new_email, new_username, password_change)
                    .await
            }
        }
    }
}
