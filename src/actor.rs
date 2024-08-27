use core::fmt;
use std::{cell::RefCell, rc::Rc};
use uuid::Uuid;

use crate::transform::Transform;

#[derive(Debug)]
pub struct Actor {
    pub name: String,
    uuid: Uuid,
    transform: RefCell<Rc<Transform>>,
}

impl Actor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            uuid: Uuid::new_v4(),
            transform: Rc::new(Transform::default()),
        }
    }

    // Getters
    pub fn uuid(&self) -> Uuid { self.uuid }
    //todo! issue here is returning an Rc makes it immutable..
    pub fn transform(&self) -> Rc<Transform> { self.transform.clone() }
}

impl fmt::Display for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Actor::{}", self.name)
    }
}