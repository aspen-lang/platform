use crate::Context;
use crate::user::User;
use uuid::Uuid;
use juniper::FieldResult;
use sha2::{Sha512, Digest};
use std::env;

#[derive(juniper::GraphQLEnum)]
enum NoOp {
    Ok,
}

pub struct Mutation;

fn hash_password(password: String) -> Vec<u8> {
    let mut hasher = Sha512::new();
    hasher.input(password);
    hasher.input(env::var("ROCKET_SECRET_KEY").unwrap_or("<<dev>>".into()));
    let hash = hasher.result();
    hash.to_vec()
}

#[juniper::object(Context = Context)]
impl Mutation {
    fn sign_up(context: &Context, username: String, email: String, password: String) -> FieldResult<User> {
        Ok(context.sign_in(User {
            id: Uuid::new_v4(),
            password: hash_password(password),
            email,
            username,
        }))
    }

    fn sign_out(context: &Context) -> NoOp {
        context.sign_out();
        NoOp::Ok
    }

    fn sign_in(context: &Context, username_or_email: String, password: String) -> User {
        context.sign_in(User {
            id: Uuid::new_v4(),
            password: hash_password(password),
            email: username_or_email.clone(),
            username: username_or_email,
        })
    }
}
