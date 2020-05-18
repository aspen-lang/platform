use std::sync::{Arc};
use std::cell::{RefCell, Ref};
use crate::user::User;

pub struct SharedContext {

}

impl SharedContext {
    pub fn new() -> SharedContext {
        SharedContext {}
    }
}

pub struct Context {
    shared: Arc<SharedContext>,
    user: RefCell<Option<User>>,
    did_sign_in: RefCell<bool>,
    did_sign_out: RefCell<bool>,
}

impl juniper::Context for Context {}

impl Context {
    pub fn new(shared: Arc<SharedContext>, user: Option<User>) -> Context {
        Context {
            shared,
            user: RefCell::new(user),
            did_sign_in: RefCell::new(false),
            did_sign_out: RefCell::new(false),
        }
    }

    pub fn did_sign_in(&self) -> bool {
        self.did_sign_in.borrow().clone()
    }

    pub fn did_sign_out(&self) -> bool {
        self.did_sign_out.borrow().clone()
    }

    pub fn user(&self) -> Ref<Option<User>> {
        self.user.borrow()
    }

    pub fn sign_in(&self, user: User) -> User {
        self.user.replace_with(|_| Some(user));
        self.did_sign_in.replace_with(|_| true);
        self.did_sign_out.replace_with(|_| false);
        self.user().as_ref().unwrap().clone()
    }

    pub fn sign_out(&self) {
        self.did_sign_in.replace_with(|_| false);
        self.did_sign_out.replace_with(|_| true);
    }
}
