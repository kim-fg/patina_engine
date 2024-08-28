pub mod prototype;
pub mod actor;
pub mod transform;
mod component;

pub mod prelude {
    pub use super::actor::*;
    pub use super::transform::*;
}