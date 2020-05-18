use crate::Context;
use crate::user::User;
use uuid::Uuid;

pub struct Mutation;

#[juniper::object(Context = Context)]
impl Mutation {
    fn sign_in(context: &Context) -> User {
        context.sign_in(User {
            id: Uuid::new_v4(),
        })
    }
}
