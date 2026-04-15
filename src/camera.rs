use glam::{Mat4, Vec3};

#[derive(Debug)]
pub struct Camera {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    aspect_ratio: f32,
    fovy_radians: f32,
    aperture: f32,
    focus_distance: f32,
    z_near: f32,
    z_far: f32,
}

impl Camera {
    pub fn new(position: Vec3, yaw: f32, pitch: f32, aspect_ratio: f32) -> Self {
        Self {
            position,
            yaw,
            pitch,
            aspect_ratio,
            fovy_radians: 45.0f32.to_radians(),
            aperture: 0.0,
            focus_distance: 1.0,
            z_near: 0.05,
            z_far: 1024.0,
        }
    }

    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize_or_zero()
    }

    pub fn up(&self) -> Vec3 {
        self.right().cross(self.forward()).normalize_or_zero()
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, self.forward(), Vec3::Y)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy_radians, self.aspect_ratio, self.z_near, self.z_far)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_inverse: [[f32; 4]; 4],
    pub projection_inverse: [[f32; 4]; 4],
    pub camera_position: [f32; 4],
    pub right: [f32; 4],
    pub up: [f32; 4],
    pub forward: [f32; 4],
    pub frame_data: [u32; 4],
}

