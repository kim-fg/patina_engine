use std::{cell::RefCell, rc::{Rc, Weak}};

use crate::actor::Actor;

pub trait Behavior {
    fn init(&self);
    fn start(&self);
    fn update(&self);
    fn destroy(&self);
}

struct Component {
    owner: RefCell<Weak<Actor>>,
    component: RefCell<Rc<dyn Behavior>>,
}
impl Component {
    pub fn new(owner: &Rc<Actor>, component: impl Behavior) -> Self {
        Self {
            owner: RefCell::new(Rc::downgrade(owner)),
            component: RefCell::new(Rc::new(component)),
        }
    }
}