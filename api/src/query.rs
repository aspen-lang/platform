use crate::Context;
use crate::user::User;

pub struct Query;

#[juniper::object(Context = Context)]
impl Query {
    fn me(context: &Context) -> Option<User> {
        context.user().clone()
    }
}
