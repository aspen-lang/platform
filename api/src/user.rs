use uuid::Uuid;
use crate::Context;

#[derive(Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password: Vec<u8>,
}

#[juniper::object(Context = Context)]
impl User {
    fn id(&self, context: &Context) -> &Uuid {
        &self.id
    }

    fn username(&self, context: &Context) -> &str {
        self.username.as_ref()
    }

    fn email(&self, context: &Context) -> Option<&str> {
        let logged_in_user = context.user();
        let logged_in_user = logged_in_user.as_ref()?;

        if logged_in_user.id != self.id {
            None
        } else {
            Some(self.email.as_str())
        }
    }
}