#![feature(decl_macro, proc_macro_hygiene, arbitrary_self_types)]

mod user;
use self::user::*;

mod context;
use self::context::*;

mod query;
use self::query::*;

mod mutation;
use self::mutation::*;

use rocket::{response::content, State, http::{Cookies, Cookie}, Outcome, Request};
use std::sync::Arc;
use rocket::request::FromRequest;

type Schema = juniper::RootNode<'static, Query, Mutation>;

#[rocket::get("/")]
fn playground() -> content::Html<String> {
    juniper_rocket::playground_source("/")
}

const AUTH_COOKIE: &str = "ASPEN_AUTH";

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> rocket::request::Outcome<Self, Self::Error> {
        let cookie = request.cookies().get_private(AUTH_COOKIE);

        match cookie {
            None => Outcome::Forward(()),
            Some(cookie) => {
                let id = cookie.value().parse();

                match id {
                    Err(_) => Outcome::Forward(()),
                    Ok(id) => {

                        Outcome::Success(User {
                            id,
                            email: "unknown@example.com".into(),
                            username: "username".into(),
                            password: vec![],
                        })
                    }
                }

            }
        }
    }
}

#[rocket::post("/", data = "<query>")]
fn graphql(
    shared_context: State<Arc<SharedContext>>,
    query: juniper_rocket::GraphQLRequest,
    schema: State<Schema>,
    user: Option<User>,
    mut cookies: Cookies,
) -> juniper_rocket::GraphQLResponse {
    let context = Context::new(shared_context.clone(), user);

    let response = query.execute(&schema, &context);

    if context.did_sign_in() {
        let user = context.user();
        let mut cookie = Cookie::new(AUTH_COOKIE, user.as_ref().unwrap().id.to_string());
        cookie.make_permanent();
        cookies.add_private(cookie)
    }

    if context.did_sign_out() {
        cookies.remove_private(Cookie::named(AUTH_COOKIE));
    }

    response
}

fn main() {
    rocket::ignite()
        .manage(Arc::new(SharedContext::new()))
        .manage(Schema::new(
            Query,
            Mutation,
        ))
        .mount(
            "/",
            rocket::routes![playground, graphql],
        )
        .launch();
}
