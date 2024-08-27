use wgpu::Extent3d;

pub trait GameInstance {
    fn title(&self) -> String;
    fn default_size(&self) -> (u32, u32);
}