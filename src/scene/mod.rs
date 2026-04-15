use std::collections::HashMap;
use glam::{Mat4, Vec2, Vec3};

pub struct MeshVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

pub struct Mesh {
    pub vertices: Vec<MeshVertex>,
}

pub struct MeshInstance {
    pub mesh: String,
    pub transform: Mat4,
}

pub struct Scene {
    pub meshes: HashMap<String, Mesh>,
    pub instances: Vec<MeshInstance>
}

impl Scene {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
            instances: Vec::new(),
        }
    }

    pub fn add_mesh(&mut self, name: String, mesh: Mesh) {
        self.meshes.insert(name, mesh);
    }

    pub fn add_mesh_instance(&mut self, mesh_instance: MeshInstance) {
        self.instances.push(mesh_instance);
    }

    pub fn add_unnamed_mesh_instance(&mut self, mesh: Mesh) {
        let mesh_name = format!("<unnamed mesh {}>", self.meshes.len());
        self.meshes.insert(mesh_name.clone(), mesh);
        self.instances.push(MeshInstance {
            mesh: mesh_name,
            transform: Mat4::IDENTITY
        });
    }
}