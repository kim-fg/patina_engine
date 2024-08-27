use cgmath::{InnerSpace, Matrix, SquareMatrix};

pub struct Camera {
    pub position: cgmath::Point3<f32>,
    pub yaw: cgmath::Rad<f32>,
    pub pitch: cgmath::Rad<f32>,
} impl Camera {
    pub fn new<
        V: Into<cgmath::Point3<f32>>, 
        Y: Into<cgmath::Rad<f32>>, 
        P: Into<cgmath::Rad<f32>>,
    >(
        position: V, 
        yaw: Y, 
        pitch: P
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calculate_matrix(&self) -> cgmath::Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();
        
        cgmath::Matrix4::look_to_rh( 
            self.position,
            cgmath::Vector3::new(
                cos_pitch * cos_yaw,
                sin_pitch,
                cos_pitch * sin_yaw
            ).normalize(),
            cgmath::Vector3::unit_y()
        )
    }
}

pub struct Projection {
    pub aspect: f32,
    pub fovy: cgmath::Rad<f32>,
    pub near: f32,
    pub far: f32,
} impl Projection {
    pub fn new<F: Into<cgmath::Rad<f32>>>(
        width: u32,
        height: u32,
        fovy: F,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            near,
            far,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calculate_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::perspective(self.fovy, self.aspect, self.near, self.far)
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
        use cgmath::SquareMatrix;
        Self {
            view_position: [0.0; 4],
            view: cgmath::Matrix4::identity().into(),
            view_proj: cgmath::Matrix4::identity().into(),
            inv_view: cgmath::Matrix4::identity().into(),
            inv_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        
        let proj = projection.calculate_matrix();
        let view = camera.calculate_matrix();
        let view_proj = projection.calculate_matrix() * camera.calculate_matrix();
        
        self.view = view.into();
        self.view_proj = view_proj.into();
        self.inv_view = view.transpose().into();
        self.inv_proj = proj.invert().unwrap().into();
    }
}