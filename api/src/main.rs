#![feature(decl_macro, proc_macro_hygiene, arbitrary_self_types)]

mod user;
use self::user::*;

mod context;
use self::context::*;

mod query;
use self::query::*;

mod mutation;
use self::mutation::*;

mod auth;

use juniper::EmptySubscription;
use juniper_rocket_async as juniper_rocket;

use rocket::{
    http::{Cookie, Cookies},
    response::content,
    State,
};
use std::sync::Arc;
use uuid::Uuid;

type Schema = juniper::RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

#[rocket::get("/")]
fn playground() -> content::Html<String> {
    juniper_rocket::playground_source("/")
}

const AUTH_COOKIE: &str = "ASPEN_AUTH";

#[rocket::post("/", data = "<query>")]
async fn graphql(
    shared_context: State<'_, Arc<SharedContext>>,
    query: juniper_rocket::GraphQLRequest,
    schema: State<'_, Schema>,
    mut cookies: Cookies<'_>,
) -> juniper_rocket::GraphQLResponse {
    let user_id = cookies.get_private(AUTH_COOKIE).and_then(|c| {
        let id: Option<Uuid> = c.value().parse().ok();
        id
    });

    let context = Context::new(
        shared_context.clone(),
        match user_id {
            None => None,
            Some(id) => shared_context.find_user(id).await,
        },
    );

    let response = query.execute(&schema, &context).await;

    if context.did_sign_in().await {
        let user = context.user().lock().await;
        let mut cookie = Cookie::new(AUTH_COOKIE, user.as_ref().unwrap().id.to_string());
        cookie.make_permanent();
        cookies.add_private(cookie)
    }

    if context.did_sign_out().await {
        cookies.remove_private(Cookie::named(AUTH_COOKIE));
    }

    response
}

#[tokio::main]
async fn main() {
    rocket::ignite()
        .manage(Arc::new(SharedContext::new().await))
        .manage(Schema::new(Query, Mutation, EmptySubscription::new()))
        .mount("/", rocket::routes![playground, graphql])
        .serve()
        .await
        .unwrap();
}
