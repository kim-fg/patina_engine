use core::fmt;
use std::{cell::RefCell, collections::HashSet, hash::Hash, rc::Rc};
use uuid::Uuid;

use crate::{component::{Behavior, Component}, transform::Transform};

#[derive(Debug, Default)]
pub struct Actor<'a> {
    pub name: String,
    uuid: Uuid,
    components: RefCell<Vec<Rc<Component<'a>>>>,
}

impl Actor<'_> {
    pub fn new(name: &str) -> Self {
        let out = Self {
            name: name.to_string(),
            uuid: Uuid::new_v4(),
            ..Default::default()
        };

        let c = out.add_component::<Transform>();
        println!("{:?}", c);

        out
    }

    // Getters
    pub fn uuid(&self) -> Uuid { self.uuid }
    //todo! issue here is returning an Rc makes it immutable..
    //pub fn transform(&self) -> Rc<Transform> { self.transform.clone() }
    
    pub fn add_component<T: Behavior + Default>(&self) -> Rc<Component> {
        let behavior = T::default();
        let component = Component::new(&self, behavior);
        let component_pointer = Rc::new(component);
        self.components.borrow_mut().push(component_pointer);
    }
}

impl fmt::Display for Actor<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Actor::{}", self.name)
    }
}