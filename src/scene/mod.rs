use glam::{Mat4, Vec4};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub position_u: Vec4,
    pub normal_v: Vec4,
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

pub enum MaterialType {
    DIFFUSE = 1,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CompactedMaterial {
    pub material_flags: u32,
    pub displacement: i32,
    pub normal_map: i32,
    pub roughness_u: i32,
    pub roughness_v: i32,
    pub reflectance: i32,
    pub eta: i32,
    pub k: i32,
}

pub struct ObjectInstance {
    pub object: usize,
    pub transform: Mat4,
    pub material: CompactedMaterial,
}

pub struct Scene {
    pub objects: Vec<SceneObject>,
    pub instances: Vec<ObjectInstance>,
    pub textures: Vec<image::DynamicImage>,
    pub camera_transformation: Mat4,
    pub camera_fov: f32,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            instances: Vec::new(),
            textures: Vec::new(),
            camera_transformation: Mat4::IDENTITY,
            camera_fov: 45.0f32.to_radians(),
        }
    }

    pub fn add_texture(&mut self, texture: image::DynamicImage) -> usize {
        self.textures.push(texture);
        self.textures.len() - 1
    }

    pub fn add_object(&mut self, object: SceneObject) -> usize {
        self.objects.push(object);
        self.objects.len() - 1
    }

    pub fn add_object_instance(&mut self, object_instance: ObjectInstance) {
        self.instances.push(object_instance);
    }
}
