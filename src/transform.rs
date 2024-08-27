use std::{cell::RefCell, ops::Index, rc::Rc};
use glam::{Quat, Vec3};

// ok so here's the problem:
// parent needs to be None or Some - Option
// if it is some, it needs a reference to a transform reference - RefCell
// multiple transforms may share the same parent - Rc

#[derive(Debug, Default)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    parent: Option<Rc<RefCell<Transform>>>,
    children: Vec<Rc<RefCell<Transform>>>,
}
impl PartialEq for Transform {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.rotation == other.rotation && self.scale == other.scale && self.parent == other.parent && self.children == other.children
    }
}

impl Transform {
    pub fn parent(&self) -> Option<Rc<RefCell<Transform>>> {
        match &self.parent {
            Some(parent_rc) => {
                Some(parent_rc.clone())
            },
            _ => None,
        }
    }

    pub fn set_parent(&mut self, parent: Option<Rc<RefCell<Transform>>>) {
        self.parent = match &parent {
            Some(parent_rc) => {
                Some(parent_rc.clone())
            },
            _ => None,
        }
    }

    pub fn children(&self) -> &Vec<Rc<RefCell<Transform>>> {
        &self.children
    }

    pub fn add_child(&mut self, child: Rc<RefCell<Transform>>) {
        if self.children.contains(&child) {
            return;
        }

        //todo! set parent of child to self

        self.children.push(child.clone())
    }
}