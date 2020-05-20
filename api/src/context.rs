use crate::user::User;
use bb8_postgres::bb8::Pool;

use bb8_postgres::PostgresConnectionManager;

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::NoTls;

pub struct SharedContext {
    pub db: Pool<PostgresConnectionManager<NoTls>>,
}

impl SharedContext {
    pub async fn new() -> SharedContext {
        let pool = Pool::builder()
            .build(PostgresConnectionManager::new(
                std::env::var("POSTGRES_URL").unwrap().parse().unwrap(),
                NoTls,
            ))
            .await
            .unwrap();

        SharedContext { db: pool }
    }
}

pub struct Context {
    pub shared: Arc<SharedContext>,
    pub user: Mutex<Option<User>>,
    pub did_sign_in: Mutex<bool>,
    pub did_sign_out: Mutex<bool>,
}

impl juniper::Context for Context {}

impl Context {
    pub fn new(shared: Arc<SharedContext>, user: Option<User>) -> Context {
        Context {
            shared,
            user: Mutex::new(user),
            did_sign_in: Mutex::new(false),
            did_sign_out: Mutex::new(false),
        }
    }
}
