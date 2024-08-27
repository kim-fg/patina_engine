pub trait Vertex {
    fn descriptor() -> wgpu::VertexBufferLayout<'static>;
}