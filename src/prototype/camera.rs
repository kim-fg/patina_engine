use glam::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
} impl Camera {
    pub fn new(
        position: Vec3, 
        yaw: f32, 
        pitch: f32,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calculate_matrix(&self) -> Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        
        Mat4::look_to_rh(
            self.position, 
            Vec3::new(
                cos_pitch * cos_yaw,
                sin_pitch,
                cos_pitch * sin_yaw,
            ).normalize(), 
            Vec3::Y,
        )
    }
}

pub struct Projection {
    pub aspect: f32,
    pub fovy: f32,
    pub z_near: f32,
    pub z_far: f32,
} impl Projection {
    pub fn new(
        width: u32,
        height: u32,
        fovy: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy,
            z_near,
            z_far,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calculate_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect, self.z_near, self.z_far)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We're using Vector4 because of the uniforms 16 byte spacing requirement
    pub view_position: [f32; 4],
    pub view: [[f32; 4]; 4],
    pub view_proj: [[f32; 4]; 4],
    pub inv_view: [[f32; 4]; 4],
    pub inv_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view: Mat4::IDENTITY.to_cols_array_2d(),
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            inv_view: Mat4::IDENTITY.to_cols_array_2d(),
            inv_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.position.extend(1.0).to_array();
        
        let proj = projection.calculate_matrix();
        let view = camera.calculate_matrix();
        let view_proj = projection.calculate_matrix() * camera.calculate_matrix();
        
        self.view = view.to_cols_array_2d();
        self.view_proj = view_proj.to_cols_array_2d();
        self.inv_view = view.transpose().to_cols_array_2d();
        self.inv_proj = proj.inverse().to_cols_array_2d();
    }
}