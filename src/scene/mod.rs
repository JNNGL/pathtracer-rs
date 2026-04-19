use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub position: Vec3,
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct SceneObject {
    pub meshes: Vec<Mesh>,
}

pub struct ObjectInstance {
    pub object: usize,
    pub transform: Mat4,
}

pub struct Scene {
    pub objects: Vec<SceneObject>,
    pub instances: Vec<ObjectInstance>,
    pub camera_transformation: Mat4,
    pub camera_fov: f32,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            instances: Vec::new(),
            camera_transformation: Mat4::IDENTITY,
            camera_fov: 45.0f32.to_radians(),
        }
    }

    pub fn add_object(&mut self, object: SceneObject) -> usize {
        self.objects.push(object);
        self.objects.len() - 1
    }

    pub fn add_object_instance(&mut self, object_instance: ObjectInstance) {
        self.instances.push(object_instance);
    }
}
