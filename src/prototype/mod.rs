mod state;
mod texture;
mod camera;
mod camera_controller;
mod instance;
mod model;
mod resources;
mod light;
mod vertex;
mod hdr;
mod rendering;

pub fn run() {
    pollster::block_on(state::run());
}