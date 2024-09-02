use std::{cell::RefCell, rc::{Rc, Weak}};
use glam::{Quat, Vec3};

use crate::component::Behavior;

#[derive(Debug)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    parent: RefCell<Weak<Transform>>,
    children: RefCell<Vec<Rc<Transform>>>,
}

impl Transform {
    pub fn parent(&self) -> Option<Rc<Transform>> {
        self.parent.borrow().upgrade()
    }

    pub fn set_parent(&self, parent: &Rc<Transform>) {
        *self.parent.borrow_mut() = Rc::downgrade(parent);
    }

    pub fn children(&self) -> &RefCell<Vec<Rc<Transform>>> {
        &self.children
    }

    pub fn add_child(&self, child: &Rc<Transform>) {
        //todo! set parent of child to self

        self.children.borrow_mut().push(child.clone())
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self { 
            position: Default::default(), 
            rotation: glam::Quat::IDENTITY, 
            scale: glam::Vec3::ONE, 
            parent: Default::default(), 
            children: Default::default() 
        }
    }
}

impl Behavior for Transform {
    fn typeid() -> &str<'static> where Self: Sized { "transform" }
}