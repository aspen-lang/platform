use uuid::Uuid;
use crate::Context;

#[derive(Clone)]
pub struct User {
    pub id: Uuid,
}

#[juniper::object(Context = Context)]
impl User {
    fn id(&self, context: &Context) -> String {
        self.id.to_string()
    }
}