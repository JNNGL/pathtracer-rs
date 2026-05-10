use glam::{Mat4, Vec4, Vec2};

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
    DIFFUSE = 0,
    DIELECTRIC = 1,
    CONDUCTOR = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SceneEnvironment {
    pub light_map: i32,
}

impl Default for SceneEnvironment {
    fn default() -> Self {
        Self {
            light_map: -1,
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CompactedMaterialPart {
    pub material_flags: u32,
    pub displacement: i32,
    pub normal_map: i32,
    pub roughness_u: i32,
    pub roughness_v: i32,
    pub reflectance: i32,
    pub eta: i32,
    pub k: i32,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CompactedMaterial {
    pub material1: CompactedMaterialPart,
    pub material2: CompactedMaterialPart,
    pub mix_factor: i32,
}

pub struct ObjectInstance {
    pub object: usize,
    pub transform: Mat4,
    pub material: CompactedMaterial,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImageInfo {
    pub uv_scale: Vec2,
    pub uv_delta: Vec2,
    pub scale: f32,
    pub gamma: f32,
    pub flags: u32,
    pub pad: u32,
}

impl ImageInfo {
    pub fn linear() -> Self {
        Self {
            uv_scale: Vec2::new(1.0, 1.0),
            uv_delta: Vec2::ZERO,
            scale: 1.0,
            gamma: 1.0,
            flags: 0,
            pad: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Texture {
    pub tex_type: u32,
    pub tex0: u32,
    pub tex1: u32,
    pub tex2: u32,
}

pub struct Scene {
    pub objects: Vec<SceneObject>,
    pub instances: Vec<ObjectInstance>,
    pub images: Vec<image::DynamicImage>,
    pub image_infos: Vec<ImageInfo>,
    pub textures: Vec<Texture>,
    pub camera_transformation: Mat4,
    pub camera_fov: f32,
    pub environment: SceneEnvironment,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            instances: Vec::new(),
            images: Vec::new(),
            image_infos: Vec::new(),
            textures: Vec::new(),
            camera_transformation: Mat4::IDENTITY,
            camera_fov: 45.0f32.to_radians(),
            environment: SceneEnvironment::default(),
        }
    }

    pub fn add_image(&mut self, image: image::DynamicImage, info: ImageInfo) -> usize {
        self.images.push(image);
        self.image_infos.push(info);
        self.images.len() - 1
    }

    pub fn add_texture(&mut self, texture: Texture) -> usize {
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
