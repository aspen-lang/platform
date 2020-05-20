use crate::user::User;
use crate::Context;

pub struct Query;

#[juniper::graphql_object(Context = Context)]
impl Query {
    async fn me(context: &Context) -> Option<User> {
        context.user().lock().await.clone()
    }
}
