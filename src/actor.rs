use core::fmt;
use std::{cell::RefCell, rc::Rc};
use uuid::Uuid;

use crate::transform::Transform;

#[derive(Debug)]
pub struct Actor {
    pub name: String,
    uuid: Uuid,
    transform: Rc<RefCell<Transform>>,
}

impl Actor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            uuid: Uuid::new_v4(),
            transform: Rc::new(RefCell::new(Transform::default())),
        }
    }

    // Getters
    pub fn uuid(&self) -> Uuid { self.uuid }
    pub fn transform(&self) -> Rc<RefCell<Transform>> { self.transform.clone() }
}

impl fmt::Display for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Actor::{}", self.name)
    }
}