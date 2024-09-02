use std::{cell::RefCell, rc::{Rc, Weak}};

use uuid::Uuid;

use crate::actor::Actor;

pub trait Behavior {
    fn typeid() -> &str<'static> where Self: Sized;
    // all these are optional per behavior
    fn init(&self) {}
    fn start(&self) {}
    fn update(&self) {}
    fn destroy(&self) {}
}

pub struct Component<'a> {
    id: Uuid,
    actor: RefCell<Weak<Actor<'a>>>,
    behavior: RefCell<Rc<dyn Behavior>>,
}
impl Component<'_> {
    pub fn new(owner: &Actor, component: impl Behavior) -> Self {
        Self {
            id: Uuid::new_v4(),
            actor: RefCell::new(owner),
            behavior: RefCell::new(Rc::new(component)),
        }
    }
}