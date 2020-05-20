use crate::Context;
use uuid::Uuid;

#[derive(Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
}

#[juniper::graphql_object(Context = Context)]
impl User {
    fn id(&self, _context: &Context) -> &Uuid {
        &self.id
    }

    fn username(&self, _context: &Context) -> &str {
        self.username.as_ref()
    }

    async fn email(&self, context: &Context) -> Option<&str> {
        let logged_in_user = context.user().lock().await;
        let logged_in_user = logged_in_user.as_ref()?;

        if logged_in_user.id != self.id {
            None
        } else {
            Some(self.email.as_str())
        }
    }
}
