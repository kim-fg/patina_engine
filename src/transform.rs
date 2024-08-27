use std::{cell::RefCell, rc::Rc};

use glam::{Quat, Vec3};

#[derive(Debug, Default)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    parent: Option<Rc<RefCell<Transform>>>
}
impl Transform {
    pub fn set_parent(&mut self, parent: Option<Rc<RefCell<Transform>>>) {
        self.parent = match &parent {
            Some(parent_rc) => {
                Some(parent_rc.clone())
            },
            _ => None,
        }
    }

    pub fn parent(&self) -> Option<Rc<RefCell<Transform>>> {
        match &self.parent {
            Some(parent_rc) => {
                Some(parent_rc.clone())
            },
            _ => None,
        }
    }
}

// ok so here's the problem:
// parent needs to be None or Some - Option
// if it is some, it needs a reference to a transform reference - RefCell
// multiple transforms may share the same parent - Rc