use glam::{EulerRot, Mat4, Vec3, Vec4Swizzles};

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

    pub fn from_transform(transform: Mat4, aspect_ratio: f32) -> Self {
        let angles = transform.to_euler(EulerRot::ZYX);
        Camera::new(
            transform.col(3).xyz(),
            angles.2,
            angles.1,
            aspect_ratio,
        )
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
        Mat4::perspective_rh(
            self.fovy_radians,
            self.aspect_ratio,
            self.z_near,
            self.z_far,
        )
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_projection_inverse: Mat4,
}
