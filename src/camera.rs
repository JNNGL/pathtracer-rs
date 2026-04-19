use std::time::Duration;

use glam::{Mat4, Quat, Vec3, Vec4Swizzles};
use winit::keyboard::KeyCode;

const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

#[derive(Debug)]
pub struct Camera {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    roll: f32,
    aspect_ratio: f32,
    fovy_radians: f32,
    aperture: f32,
    focus_distance: f32,
    z_near: f32,
    z_far: f32,
}

fn basis_from_angles(yaw: f32, pitch: f32, roll: f32) -> (Vec3, Vec3, Vec3) {
    let forward = Vec3::new(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    ).normalize();

    let base_right = forward.cross(Vec3::Y).normalize_or_zero();
    let base_up = base_right.cross(forward).normalize_or_zero();
    let bank = Quat::from_axis_angle(forward, roll);

    (forward, (bank * base_right).normalize_or_zero(), (bank * base_up).normalize_or_zero())
}

fn extract_angles(transform: Mat4) -> (f32, f32, f32) {
    let right = transform.col(0).xyz().normalize_or_zero();
    let forward = (-transform.col(2).xyz()).normalize_or_zero();

    let yaw = forward.z.atan2(forward.x);
    let pitch = forward.y.clamp(-1.0, 1.0).asin();

    let world_up = Vec3::Y;
    let ref_right = forward.cross(world_up).normalize_or_zero();

    let roll = if ref_right.length_squared() > 0.0 {
        forward
            .dot(ref_right.cross(right))
            .atan2(ref_right.dot(right))
    } else {
        0.0
    };

    (yaw, pitch, roll)
}

fn camera_to_world(camera_from_world: Mat4) -> Mat4 {
    let camera_to_world = camera_from_world.inverse();
    let position = camera_to_world.col(3).xyz();
    let right = camera_to_world.col(0).xyz().normalize_or_zero();
    let forward = camera_to_world.col(2).xyz().normalize_or_zero();

    let up = right.cross(forward).normalize_or_zero();
    Mat4::from_cols(
        right.extend(0.0),
        up.extend(0.0),
        -forward.extend(0.0),
        position.extend(1.0),
    )
}

impl Camera {
    pub fn new(position: Vec3, yaw: f32, pitch: f32, roll: f32, aspect_ratio: f32) -> Self {
        Self {
            position,
            yaw,
            pitch,
            roll,
            aspect_ratio,
            fovy_radians: 45.0f32.to_radians(),
            aperture: 0.0,
            focus_distance: 1.0,
            z_near: 0.05,
            z_far: 1024.0,
        }
    }

    pub fn from_transform(transform: Mat4, aspect_ratio: f32) -> Self {
        let mut camera = Camera::new(Vec3::ZERO, 0.0, 0.0, 0.0, aspect_ratio);
        camera.set_transform(&transform);
        camera
    }

    pub fn set_transform(&mut self, transform: &Mat4) {
        let camera_to_world = camera_to_world(*transform);
        let angles = extract_angles(camera_to_world);
        self.position = camera_to_world.col(3).xyz();
        self.yaw = angles.0;
        self.pitch = angles.1;
        self.roll = angles.2;
    }

    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.aspect_ratio = width as f32 / height as f32;
    }

    pub fn set_fov(&mut self, fovy_radians: f32) {
        self.fovy_radians = fovy_radians;
    }

    pub fn translate(&mut self, offset: Vec3) {
        self.position += offset;
    }

    pub fn rotate(&mut self, yaw_delta: f32, pitch_delta: f32) {
        self.yaw += yaw_delta;
        self.pitch = (self.pitch + pitch_delta).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    pub fn forward(&self) -> Vec3 {
        basis_from_angles(self.yaw, self.pitch, self.roll).0
    }

    pub fn right(&self) -> Vec3 {
        basis_from_angles(self.yaw, self.pitch, self.roll).1
    }

    pub fn up(&self) -> Vec3 {
        basis_from_angles(self.yaw, self.pitch, self.roll).2
    }

    pub fn camera_to_world_matrix(&self) -> Mat4 {
        let (forward, right, up) = basis_from_angles(self.yaw, self.pitch, self.roll);
        Mat4::from_cols(
            right.extend(0.0),
            up.extend(0.0),
            -forward.extend(0.0),
            self.position.extend(1.0),
        )
    }

    pub fn view_matrix(&self) -> Mat4 {
        self.camera_to_world_matrix().inverse()
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(
            self.fovy_radians,
            self.aspect_ratio,
            self.z_near,
            self.z_far,
        )
    }

    pub fn uniform_data(&self) -> CameraUniform {
        CameraUniform {
            view_inverse: self.camera_to_world_matrix(),
            projection_inverse: self.projection_matrix().inverse(),
        }
    }
}

#[derive(Debug)]
pub struct CameraController {
    move_speed: f32,
    look_sensitivity: f32,
    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,
    mouse_captured: bool,
}

impl CameraController {
    pub fn new(move_speed: f32, look_sensitivity: f32) -> Self {
        Self {
            move_speed,
            look_sensitivity,
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            move_up: false,
            move_down: false,
            mouse_captured: false,
        }
    }

    pub fn set_mouse_captured(&mut self, mouse_captured: bool) {
        self.mouse_captured = mouse_captured;
    }

    pub fn mouse_captured(&self) -> bool {
        self.mouse_captured
    }

    pub fn set_key_pressed(&mut self, key: KeyCode, is_pressed: bool) -> bool {
        let target = match key {
            KeyCode::KeyW => Some(&mut self.move_forward),
            KeyCode::KeyS => Some(&mut self.move_backward),
            KeyCode::KeyA => Some(&mut self.move_left),
            KeyCode::KeyD => Some(&mut self.move_right),
            KeyCode::Space => Some(&mut self.move_up),
            KeyCode::ShiftLeft | KeyCode::ShiftRight => Some(&mut self.move_down),
            _ => None,
        };

        if let Some(target) = target {
            *target = is_pressed;
            true
        } else {
            false
        }
    }

    pub fn clear_input(&mut self) {
        self.move_forward = false;
        self.move_backward = false;
        self.move_left = false;
        self.move_right = false;
        self.move_up = false;
        self.move_down = false;
    }

    pub fn process_mouse_motion(&self, camera: &mut Camera, delta_x: f64, delta_y: f64) -> bool {
        if !self.mouse_captured {
            return false;
        }

        camera.rotate(
            delta_x as f32 * self.look_sensitivity,
            -(delta_y as f32) * self.look_sensitivity,
        );
        true
    }

    pub fn update_camera(&self, camera: &mut Camera, delta_time: Duration) -> bool {
        let delta_seconds = delta_time.as_secs_f32();
        if delta_seconds <= 0.0 {
            return false;
        }

        let forward = camera.forward();
        let right = camera.right();

        let mut movement = Vec3::ZERO;
        if self.move_forward {
            movement += forward;
        }
        if self.move_backward {
            movement -= forward;
        }
        if self.move_right {
            movement += right;
        }
        if self.move_left {
            movement -= right;
        }
        if self.move_up {
            movement += Vec3::Y;
        }
        if self.move_down {
            movement -= Vec3::Y;
        }

        if movement == Vec3::ZERO {
            return false;
        }

        camera.translate(movement.normalize() * self.move_speed * delta_seconds);
        true
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_inverse: Mat4,
    pub projection_inverse: Mat4,
}
