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

    async fn me(
        context: &Context,
        email: Option<String>,
        username: Option<String>,
        password: Option<PasswordChange>,
    ) -> FieldResult<User> {
        Ok(context.update_account(email, username, password).await?)
    }
}

#[derive(juniper::GraphQLInputObject)]
pub struct PasswordChange {
    pub current: String,
    pub new: String,
}
