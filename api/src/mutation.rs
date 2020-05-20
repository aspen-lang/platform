use crate::user::User;
use crate::Context;
use juniper::FieldResult;

#[derive(juniper::GraphQLEnum)]
enum NoOp {
    Ok,
}

pub struct Mutation;

#[juniper::graphql_object(Context = Context)]
impl Mutation {
    async fn sign_up(
        context: &Context,
        username: String,
        email: String,
        password: String,
    ) -> FieldResult<User> {
        Ok(context.sign_up(&username, &email, &password).await?)
    }

    async fn sign_out(context: &Context) -> NoOp {
        context.sign_out().await;
        NoOp::Ok
    }

    async fn sign_in(
        context: &Context,
        username_or_email: String,
        password: String,
    ) -> FieldResult<User> {
        Ok(context.sign_in(&username_or_email, &password).await?)
    }

    async fn remove_account(context: &Context, password: String) -> FieldResult<NoOp> {
        context.remove_account(&password).await?;
        Ok(NoOp::Ok)
    }
}
