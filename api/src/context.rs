use crate::user::User;
use bb8_postgres::bb8::Pool;

use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::Error as DbError;

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::NoTls;

pub struct SharedContext {
    pub db: Pool<PostgresConnectionManager<NoTls>>,
}

#[derive(Debug)]
struct ErrorSink;

impl bb8_postgres::bb8::ErrorSink<DbError> for ErrorSink {
    fn sink(&self, error: DbError) {
        log::error!("{:?}", error);
    }

    fn boxed_clone(&self) -> Box<dyn bb8_postgres::bb8::ErrorSink<DbError>> {
        Box::new(ErrorSink)
    }
}

impl SharedContext {
    pub async fn new() -> SharedContext {
        let pool = Pool::builder()
            .error_sink(Box::new(ErrorSink))
            .build(PostgresConnectionManager::new(
                std::env::var("POSTGRES_URL").unwrap().parse().unwrap(),
                NoTls,
            ))
            .await
            .unwrap();

        {
            let client = pool.dedicated_connection().await.unwrap();

            client.query_one("SELECT 1", &[]).await.unwrap();
        }

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
