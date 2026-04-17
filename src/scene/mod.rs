use glam::{Mat4, Vec2, Vec3};

#[derive(Debug, Clone, Copy)]
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
    pub meshes: Vec<SceneObject>,
    pub instances: Vec<ObjectInstance>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            instances: Vec::new(),
        }
    }

    pub fn add_mesh(&mut self, object: SceneObject) -> usize {
        self.meshes.push(object);
        self.meshes.len() - 1
    }

    pub fn add_mesh_instance(&mut self, mesh_instance: ObjectInstance) {
        self.instances.push(mesh_instance);
    }
}
