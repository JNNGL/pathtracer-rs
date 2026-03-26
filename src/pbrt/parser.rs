use std::collections::HashMap;

use crate::pbrt::Tokenizer;

#[derive(Debug, Clone)]
pub enum ParameterValue {
    Integer(Vec<i32>),
    Float(Vec<f32>),
    Point2(Vec<glam::Vec2>),
    Vector2(Vec<glam::Vec2>),
    Point3(Vec<glam::Vec3>),
    Vector3(Vec<glam::Vec3>),
    Normal3(Vec<glam::Vec3>),
    Spectrum(Spectrum),
    RGB(Vec<glam::Vec3>),
    Bool(Vec<bool>),
    String(Vec<String>),
    Texture(String),
}

#[derive(Debug, Clone)]
pub struct NamedParameter {
    pub name: String,
    pub value: ParameterValue,
}

#[derive(Debug)]
pub enum ActiveTransform {
    StartTime,
    EndTime,
    All
}

#[derive(Debug)]
pub enum ApertureShape {
    Circular,
    Gaussian,
    Square,
    Pentagon,
    Star,
    Custom(String),
}

#[derive(Debug)]
pub enum SphericalMapping {
    Equirectangular,
    Equalarea,
}

#[derive(Debug)]
pub enum CameraType {
    Orthographic {
        frame_aspect_ratio: Option<f32>,
        screen_window: Option<f32>,
        lens_radius: f32,
        focal_distance: f32,
    },
    Perspective {
        frame_aspect_ratio: Option<f32>,
        screen_window: Option<f32>,
        lens_radius: f32,
        focal_distance: f32,
        fov: f32,
    },
    Spherical {
        mapping: SphericalMapping,
    },
    Realistic {
        lens_file: String,
        aperture_diameter: f32,
        focus_distance: f32,
        aperture: ApertureShape,
    },
}

#[derive(Debug)]
pub enum SamplerType {
    Halton,
    Independent,
    PaddedSobol,
    Sobol,
    Stratified,
    ZSobol,
}

#[derive(Debug, Clone, Default)]
pub enum ColorSpace {
    Aces2065_1,
    Rec2020,
    DciP3,
    #[default]
    Srgb
}

#[derive(Debug, Clone)]
pub enum Spectrum {
    Piecewise(Vec<glam::Vec2>),
    Blackbody(f32),
    RGB(glam::Vec3),
    Constant(f32),
    Named(String),
}

#[derive(Debug)]
pub enum PixelFilter {
    Box { x_radius: f32, y_radius: f32 },
    Gaussian { x_radius: f32, y_radius: f32, sigma: f32 },
    Mitchell { x_radius: f32, y_radius: f32, b: f32, c: f32 },
    LanczosSinc { x_radius: f32, y_radius: f32, tau: f32 },
    Triangle { x_radius: f32, y_radius: f32 },
}

#[derive(Debug)]
pub enum AttributeTarget {
    Shape,
    Light,
    Material,
    Medium,
    Texture,
}

#[derive(Debug)]
pub enum CurveBasis {
    Bezier,
    BSpline,
}

#[derive(Debug)]
pub enum CurveVariant {
    Flat,
    Cylinder,
    Ribbon,
}

#[derive(Debug)]
pub enum Shape {
    Curve {
        points: Option<[glam::Vec3; 4]>,
        basis: CurveBasis,
        degree: i32,
        variant: CurveVariant,
        normals: Option<[glam::Vec3; 2]>,
        start_width: f32,
        end_width: f32,
        split_depth: i32,
    },
    Cylinder {
        radius: f32,
        z_min: f32,
        z_max: f32,
        phi_max: f32,
    },
    Disk {
        height: f32,
        radius: f32,
        inner_radius: f32,
        phi_max: f32,
    },
    Sphere {
        radius: f32,
        z_min: f32,
        z_max: f32,
        phi_max: f32,
    },
    TriangleMesh {
        indices: Option<Vec<i32>>,
        vertices: Vec<glam::Vec3>,
        normals: Option<Vec<glam::Vec3>>,
        tangents: Option<Vec<glam::Vec3>>,
        uvs: Option<Vec<glam::Vec2>>,
    },
    PlyMesh {
        filename: String,
        displacement: Option<TextureRef>,
        edge_length: f32,
    },
}

#[derive(Debug)]
pub enum Light {
    Distant {
        illuminant: Spectrum,
        from: glam::Vec3,
        to: glam::Vec3,
    },
    Goniometric {
        filename: String,
        illuminant: Spectrum,
    },
    Infinite {
        filename: Option<String>,
        portal: Option<[glam::Vec3; 4]>,
        illuminant: Spectrum,
    },
    Point {
        illuminant: Spectrum,
        from: glam::Vec3,
    },
    Projection {
        illuminant: Spectrum,
        fov: f32,
        filename: String,
    },
    Spotlight {
        illuminant: Spectrum,
        from: glam::Vec3,
        to: glam::Vec3,
        cone_angle: f32,
        cone_delta_angle: f32,
    },
}

#[allow(unused)]
#[derive(Debug, Clone, Default)]
pub struct BumpNormalMap {
    displacement: Option<TextureRef>,
    normal_map: Option<String>,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Roughness {
    u: f32,
    v: f32,
    remap: bool,
}

#[derive(Debug, Clone)]
pub struct Coating {
    pub albedo: TextureRef,
    pub asymmetry: TextureRef,
    pub max_depth: i32,
    pub samples: i32,
    pub thickness: f32,
}

#[derive(Debug, Clone)]
pub enum Material {
    CoatedDiffuse {
        normal: BumpNormalMap,
        roughness: Roughness,
        coating: Coating,
        reflectance: TextureRef,
    },
    CoatedConductor {
        normal: BumpNormalMap,
        interface_roughness: Roughness,
        conductor_roughness: Roughness,
        coating: Coating,
        eta: Spectrum,
        k: Spectrum,
        reflectance: Option<Spectrum>,
    },
    Conductor {
        normal: BumpNormalMap,
        roughness: Roughness,
        eta: TextureRef,
        k: TextureRef,
        reflectance: Option<TextureRef>,
    },
    Dielectric {
        normal: BumpNormalMap,
        roughness: Roughness,
        eta: TextureRef,
    },
    Diffuse {
        normal: BumpNormalMap,
        reflectance: TextureRef,
    },
    DiffuseTransmission {
        normal: BumpNormalMap,
        reflectance: TextureRef,
        transmittance: TextureRef,
        scale: TextureRef,
    },
    Hair {
        normal: BumpNormalMap,
        sigma_a: Option<TextureRef>,
        reflectance: Option<TextureRef>,
        eumelanin: Option<TextureRef>,
        pheomelanin: Option<TextureRef>,
        eta: TextureRef,
        beta_m: TextureRef,
        beta_n: TextureRef,
        alpha: TextureRef,
    },
    Interface,
    Measured {
        normal: BumpNormalMap,
        filename: String,
    },
    Mix {
        materials: [String; 2],
        amount: TextureRef,
    },
    Subsurface {
        normal: BumpNormalMap,
        roughness: Roughness,
        eta: TextureRef,
        asymmetry: TextureRef,
        mfp: Option<TextureRef>,
        name: Option<String>,
        reflectance: Option<TextureRef>,
        sigma_a: TextureRef,
        sigma_s: TextureRef,
        scale: f32,
    },
}

impl Default for Material {
    fn default() -> Self {
        Material::Diffuse {
            normal: BumpNormalMap::default(),
            reflectance: TextureRef::Float(0.5),
        }
    }
}

#[derive(Debug)]
pub enum TextureType {
    Spectrum,
    Float,
}

#[derive(Debug)]
pub enum TextureMapping {
    Uv { scale: glam::Vec2, delta: glam::Vec2 },
    Spherical,
    Cylindrical,
    Planar { delta: glam::Vec2, v1: glam::Vec3, v2: glam::Vec3 },
}

#[derive(Debug)]
pub enum FilterMode {
    Bilinear,
    Ewa,
    Trilinear,
    Point,
}

#[derive(Debug)]
pub enum TextureEncoding {
    Srgb,
    Linear,
    Gamma(f32),
}

#[derive(Debug)]
pub enum TextureWrap {
    Repeat,
    Black,
    Clamp,
}

#[derive(Debug)]
pub enum Texture {
    BilinearInterpolation {
        v00: TextureRef,
        v01: TextureRef,
        v10: TextureRef,
        v11: TextureRef,
    },
    Checkerboard {
        dimension: i32,
        texture1: TextureRef,
        texture2: TextureRef,
    },
    Constant {
        value: TextureRef,
    },
    DirectionMix {
        texture1: TextureRef,
        texture2: TextureRef,
        direction: glam::Vec3,
    },
    Dots {
        inside: TextureRef,
        outside: TextureRef,
    },
    Fbm {
        octaves: i32,
        roughness: f32,
    },
    Wrinkled {
        octaves: i32,
        roughness: f32,
    },
    Windy {
        octaves: i32,
        roughness: f32,
    },
    ImageMap {
        filename: String,
        wrap: TextureWrap,
        max_anisotropy: f32,
        filter: FilterMode,
        encoding: TextureEncoding,
        scale: f32,
        invert: bool,
    },
    Marble {
        octaves: i32,
        roughness: f32,
        scale: f32,
        variation: f32,
    },
    Mix {
        texture1: TextureRef,
        texture2: TextureRef,
        amount: TextureRef,
    },
    Ptex {
        encoding: TextureEncoding,
        filename: String,
        scale: f32,
    },
    Scale {
        texture: TextureRef,
        scale: TextureRef,
    },
}

#[derive(Debug, Clone)]
pub enum TextureRef {
    Float(f32),
    RGB(glam::Vec3),
    Spectrum(Spectrum),
    Named(String),
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct AreaLightSource {
    filename: Option<String>,
    illuminant: Spectrum,
    two_sided: bool,
}

#[derive(Debug)]
pub enum Directive {
    Identity,
    Translate(glam::Vec3),
    Scale(glam::Vec3),
    Rotate { angle: f32, axis: glam::Vec3 },
    LookAt { eye: glam::Vec3, look: glam::Vec3, up: glam::Vec3 },
    CoordinateSystem { name: String },
    CoordSysTransform { name: String },
    Transform(glam::Mat4),
    ConcatTransform(glam::Mat4),
    TransformTimes { start: f32, end: f32 },
    ActiveTransform(ActiveTransform),
    Include(String),
    Import(String),
    Option(NamedParameter),
    Camera { camera: CameraType, shutter_open: f32, shutter_close: f32 },
    Sampler { sampler: SamplerType, seed: i32 },
    ColorSpace(ColorSpace),
    Film { x_resolution: i32, y_resolution: i32,
        crop_window: [glam::Vec2; 2], pixel_bounds: [glam::IVec2; 2],
        diagonal: f32, filename: String, iso: f32, white_balance: f32,
        sensor: String },
    PixelFilter { filter: PixelFilter },
    WorldBegin,
    AttributeBegin,
    AttributeEnd,
    ReverseOrientation,
    Attribute { target: AttributeTarget, parameters: ParameterDictionary },
    Shape { shape: Shape, alpha: TextureRef },
    ObjectBegin { name: String },
    ObjectEnd,
    ObjectInstance { name: String },
    LightSource { light: Light, illuminance: Option<f32>, scale: f32 },
    AreaLightSource(AreaLightSource),
    Material(Material),
    MakeNamedMaterial { name: String, material: Material },
    NamedMaterial { name: String },
    Texture { name: String, texture_type: TextureType, texture: Texture, mapping: TextureMapping },
    Unimplemented(&'static str),
}

fn is_quoted_string(str: &String) -> bool {
    str.starts_with("\"") && str.ends_with("\"")
}

fn unquote_string(str: &String) -> Result<String, &'static str> {
    if !is_quoted_string(str) {
        return Err("expected quoted string");
    }

    let str = str.strip_prefix("\"").unwrap();
    Ok(str.strip_suffix("\"").unwrap().to_string())
}

trait ParseValue<T> {
    fn parse(token: &String) -> Result<T, &'static str>;
}

impl ParseValue<i32> for i32 {
    fn parse(token: &String) -> Result<i32, &'static str> {
        match token.parse::<i32>() {
            Ok(v) => Ok(v),
            Err(_) => Err("failed to parse int"),
        }
    }
}

impl ParseValue<f32> for f32 {
    fn parse(token: &String) -> Result<f32, &'static str> {
        match token.parse::<f32>() {
            Ok(v) => Ok(v),
            Err(_) => Err("failed to parse float"),
        }
    }
}

impl ParseValue<bool> for bool {
    fn parse(token: &String) -> Result<bool, &'static str> {
        match token.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err("failed to parse bool"),
        }
    }
}

impl ParseValue<String> for String {
    fn parse(token: &String) -> Result<String, &'static str> {
        Ok(unquote_string(token)?)
    }
}

fn parse_vector<T: ParseValue<T>>(tokenizer: &mut Tokenizer)
    -> Result<Vec<T>, &'static str> {
    let t = tokenizer.expect_token()?;

    if t.token == "[" {
        let mut vec = Vec::<T>::new();

        loop {
            let t = tokenizer.expect_token()?;
            if t.token == "]" {
                break;
            }

            vec.push(T::parse(&t.token)?);
        }

        Ok(vec)
    } else {
        Ok(vec![T::parse(&t.token)?])
    }
}

fn parse_named_parameter(tokenizer: &mut Tokenizer)
    -> Result<Option<NamedParameter>, &'static str> {
    let token = match tokenizer.next_token()? {
        Some(token) => token,
        None => return Ok(None),
    };

    let token = match unquote_string(&token.token) {
        Ok(v) => v,
        Err(_) => {
            tokenizer.push_token(token);
            return Ok(None);
        },
    };

    let (param_type, name) = token.split_once(" ")
        .ok_or("invalid parameter syntax")?;

    match param_type {
        "integer" => Ok(Some(NamedParameter {
            name: String::from(name),
            value: ParameterValue::Integer(parse_vector::<i32>(tokenizer)?)
        })),
        "float" => Ok(Some(NamedParameter {
            name: String::from(name),
            value: ParameterValue::Float(parse_vector::<f32>(tokenizer)?)
        })),
        "bool" => Ok(Some(NamedParameter {
            name: String::from(name),
            value: ParameterValue::Bool(parse_vector::<bool>(tokenizer)?)
        })),
        "string" => Ok(Some(NamedParameter {
            name: String::from(name),
            value: ParameterValue::String(parse_vector::<String>(tokenizer)?)
        })),
        "point2" => Ok(Some(NamedParameter {
            name: String::from(name),
            value: ParameterValue::Point2(
                parse_vector::<f32>(tokenizer)?
                    .chunks_exact(2)
                    .map(|chunk| glam::vec2(chunk[0], chunk[1]))
                    .collect()
            )
        })),
        "vector2" => Ok(Some(NamedParameter {
            name: String::from(name),
            value: ParameterValue::Vector2(
                parse_vector::<f32>(tokenizer)?.chunks_exact(2)
                    .map(|chunk| glam::vec2(chunk[0], chunk[1]))
                    .collect()
            )
        })),
        "point3" | "point" => Ok(Some(NamedParameter {
            name: String::from(name),
            value: ParameterValue::Point3(
                parse_vector::<f32>(tokenizer)?.chunks_exact(3)
                    .map(|chunk| glam::vec3(chunk[0], chunk[1], chunk[2]))
                    .collect()
            )
        })),
        "vector3" | "vector" => Ok(Some(NamedParameter {
            name: String::from(name),
            value: ParameterValue::Vector3(
                parse_vector::<f32>(tokenizer)?.chunks_exact(3)
                    .map(|chunk| glam::vec3(chunk[0], chunk[1], chunk[2]))
                    .collect()
            )
        })),
        "normal3" | "normal" => Ok(Some(NamedParameter {
            name: String::from(name),
            value: ParameterValue::Normal3(
                parse_vector::<f32>(tokenizer)?.chunks_exact(3)
                    .map(|chunk| glam::vec3(chunk[0], chunk[1], chunk[2]))
                    .collect()
            )
        })),
        "spectrum" => {
            let t = tokenizer.expect_token()?;

            if t.token == "[" {
                tokenizer.push_token(t);

                Ok(Some(NamedParameter {
                    name: String::from(name),
                    value: ParameterValue::Spectrum(Spectrum::Piecewise(
                        parse_vector::<f32>(tokenizer)?.chunks_exact(2)
                            .map(|chunk| glam::vec2(chunk[0], chunk[1]))
                            .collect()
                    ))
                }))
            } else {
                tokenizer.push_token(t);

                Ok(Some(NamedParameter {
                    name: String::from(name),
                    value: ParameterValue::Spectrum(
                        Spectrum::Named(expect_string(tokenizer)?))
                }))
            }
        },
        "rgb" => Ok(Some(NamedParameter {
            name: String::from(name),
            value: ParameterValue::RGB(
                parse_vector::<f32>(tokenizer)?.chunks_exact(3)
                    .map(|chunk| glam::vec3(chunk[0], chunk[1], chunk[2]))
                    .collect()
            )
        })),
        "blackbody" => Ok(Some(NamedParameter {
            name: String::from(name),
            value: ParameterValue::Spectrum(Spectrum::Blackbody(expect_float(tokenizer)?))
        })),
        "texture" => Ok(Some(NamedParameter {
            name: String::from(name),
            value: ParameterValue::Texture(expect_string(tokenizer)?),
        })),
        _ => Err("invalid parameter type"),
    }
}

#[derive(Debug, Clone, Default)]
pub struct ParameterDictionary {
    parameters: HashMap<String, NamedParameter>,
}

macro_rules! typed_getters {
    ($many:ident, $one:ident, $variant:ident, $ty:ty) => {
        #[allow(unused)]
        fn $many<'a>(&'a self, name: &str, fallback: Option<&'a Self>)
            -> Result<Option<&'a Vec<$ty>>, &'static str> {
            self.get_vec(name, fallback, |v| match v {
                ParameterValue::$variant(x) => Some(x),
                _ => None,
            })
        }

        #[allow(unused)]
        fn $one<'a>(&'a self, name: &str, fallback: Option<&'a Self>)
            -> Result<Option<$ty>, &'static str> {
            self.get_first(name, fallback, |v| match v {
                ParameterValue::$variant(x) => Some(x),
                _ => None,
            }).map(|v| v.map(|f| *f))
        }
    };
}

impl ParameterDictionary {
    fn get<'a>(&'a self, name: &str, fallback: Option<&'a Self>)
        -> Option<&'a NamedParameter> {
        self.parameters
            .get(name)
            .or_else(|| fallback.and_then(|fb| fb.parameters.get(name)))
    }

    fn get_vec<'a, T>(
        &'a self, name: &str, fallback: Option<&'a Self>,
        cast: impl FnOnce(&'a ParameterValue) -> Option<&'a Vec<T>>,
    ) -> Result<Option<&'a Vec<T>>, &'static str> {
        match self.get(name, fallback).map(|p| &p.value) {
            Some(v) => cast(v).map(Some).ok_or("wrong parameter type"),
            None => Ok(None),
        }
    }

    fn get_first<'a, T>(
        &'a self, name: &str, fallback: Option<&'a Self>,
        cast: impl FnOnce(&'a ParameterValue) -> Option<&'a Vec<T>>,
    ) -> Result<Option<&'a T>, &'static str> {
        match self.get_vec(name, fallback, cast)? {
            Some(v) => v.first().map(Some).ok_or("expected at least one element"),
            None => Ok(None),
        }
    }

    typed_getters!(get_integers, get_integer, Integer, i32);
    typed_getters!(get_floats, get_float, Float, f32);
    typed_getters!(get_bools, get_bool, Bool, bool);
    typed_getters!(get_points2, get_point2, Point2, glam::Vec2);
    typed_getters!(get_vectors2, get_vector2, Vector2, glam::Vec2);
    typed_getters!(get_points3, get_point3, Point3, glam::Vec3);
    typed_getters!(get_vectors3, get_vector3, Vector3, glam::Vec3);
    typed_getters!(get_normals3, get_normal3, Normal3, glam::Vec3);
    typed_getters!(get_rgbs, get_rgb, RGB, glam::Vec3);

    fn get_strings<'a>(&'a self, name: &str, fallback: Option<&'a Self>)
        -> Result<Option<&'a Vec<String>>, &'static str> {
        self.get_vec(name, fallback, |v| match v {
            ParameterValue::String(x) => Some(x),
            _ => None,
        })
    }

    fn get_string<'a>(&'a self, name: &str, fallback: Option<&'a Self>)
        -> Result<Option<&'a String>, &'static str> {
        self.get_first(name, fallback, |v| match v {
            ParameterValue::String(x) => Some(x),
            _ => None,
        })
    }

    fn get_texture_ref<'a>(&'a self, name: &str, fallback: Option<&'a Self>)
        -> Result<Option<TextureRef>, &'static str> {
        let parameter = match self.get(name, fallback) {
            Some(v) => v,
            None => return Ok(None),
        };

        match &parameter.value {
            ParameterValue::Texture(texture) =>
                Ok(Some(TextureRef::Named(texture.clone()))),
            ParameterValue::Float(value) => Ok(Some(TextureRef::Float(
                *value.first().ok_or("expected at least one element")?))),
            ParameterValue::RGB(value) => Ok(Some(TextureRef::RGB(
                *value.first().ok_or("expected at least one element")?))),
            ParameterValue::Spectrum(value) => Ok(Some(
                TextureRef::Spectrum(value.clone()))),
            _ => Err("invalid parameter type"),
        }
    }

    fn get_spectrum<'a>(&'a self, name: &str, fallback: Option<&'a Self>)
        -> Result<Option<Spectrum>, &'static str> {
        let parameter = match self.get(name, fallback) {
            Some(v) => v,
            None => return Ok(None),
        };

        match &parameter.value {
            ParameterValue::Float(value) => Ok(Some(Spectrum::Constant(
                *value.first().ok_or("expected at least one element")?))),
            ParameterValue::RGB(value) => Ok(Some(Spectrum::RGB(
                *value.first().ok_or("expected at least one element")?))),
            ParameterValue::Spectrum(value) => Ok(Some(value.clone())),
            _ => Err("invalid parameter type"),
        }
    }
}

fn parse_parameter_list(tokenizer: &mut Tokenizer) -> Result<ParameterDictionary, &'static str> {
    let mut parameters = HashMap::<String, NamedParameter>::new();

    loop {
        match parse_named_parameter(tokenizer)? {
            Some(p) => parameters.insert(p.name.to_string(), p),
            None => return Ok(ParameterDictionary { parameters }),
        };
    }
}

fn expect_float(tokenizer: &mut Tokenizer) -> Result<f32, &'static str> {
    Ok(f32::parse(&tokenizer.expect_token()?.token)?)
}

fn expect_vec3(tokenizer: &mut Tokenizer) -> Result<glam::Vec3, &'static str> {
    Ok(glam::vec3(
        expect_float(tokenizer)?,
        expect_float(tokenizer)?,
        expect_float(tokenizer)?,
    ))
}

fn expect_string(tokenizer: &mut Tokenizer) -> Result<String, &'static str> {
    Ok(unquote_string(&tokenizer.expect_token()?.token)?)
}

fn expect_mat4(tokenizer: &mut Tokenizer) -> Result<glam::Mat4, &'static str> {
    Ok(glam::Mat4::from_cols_slice(parse_vector::<f32>(tokenizer)?.as_slice()))
}

fn parse_bump_normal_map(
    state: &ParseState,
    parameters: &ParameterDictionary,
) -> Result<BumpNormalMap, &'static str> {
    Ok(BumpNormalMap {
        displacement: parameters.get_texture_ref("displacement", Some(&state.material_attributes))?,
        normal_map: parameters.get_string("normalmap", Some(&state.material_attributes))?.map(|s| s.clone()),
    })
}

macro_rules! parse_roughness {
    ($state:ident, $parameters:ident, $prefix:literal) => {
        {
            let roughness = $parameters.get_float(concat!($prefix, "roughness"), Some(&$state.material_attributes))?.unwrap_or(0.0);

            Ok(Roughness {
                u: $parameters.get_float(concat!($prefix, "uroughness"), Some(&$state.material_attributes))?.unwrap_or(roughness),
                v: $parameters.get_float(concat!($prefix, "vroughness"), Some(&$state.material_attributes))?.unwrap_or(roughness),
                remap: $parameters.get_bool(concat!($prefix, "remaproughness"), Some(&$state.material_attributes))?.unwrap_or(true),
            })
        }
    }
}

fn parse_coating(
    state: &ParseState,
    parameters: &ParameterDictionary,
) -> Result<Coating, &'static str> {
    Ok(Coating {
        albedo: parameters.get_texture_ref("albedo", Some(&state.material_attributes))?.unwrap_or(TextureRef::Float(0.0)),
        asymmetry: parameters.get_texture_ref("g", Some(&state.material_attributes))?.unwrap_or(TextureRef::Float(0.0)),
        max_depth: parameters.get_integer("maxdepth", Some(&state.material_attributes))?.unwrap_or(10),
        samples: parameters.get_integer("nsamples", Some(&state.material_attributes))?.unwrap_or(1),
        thickness: parameters.get_float("thickness", Some(&state.material_attributes))?.unwrap_or(0.01),
    })
}

fn parse_material(state: &ParseState, tokenizer: &mut Tokenizer) -> Result<Material, &'static str> {
    let material = expect_string(tokenizer)?;
    let parameters = parse_parameter_list(tokenizer)?;

    match material.as_str() {
        "coateddiffuse" => Ok(Material::CoatedDiffuse {
            normal: parse_bump_normal_map(&state, &parameters)?,
            roughness: parse_roughness!(state, parameters, "")?,
            reflectance: parameters.get_texture_ref("reflectance", Some(&state.material_attributes))?
                .unwrap_or(TextureRef::Float(0.5)),
            coating: parse_coating(&state, &parameters)?,
        }),
        "coatedconductor" => Ok(Material::CoatedConductor {
            normal: parse_bump_normal_map(&state, &parameters)?,
            interface_roughness: parse_roughness!(state, parameters, "interface.")?,
            conductor_roughness: parse_roughness!(state, parameters, "conductor.")?,
            eta: parameters.get_spectrum("conductor.eta", Some(&state.material_attributes))?
                .unwrap_or_else(|| Spectrum::Named("metal-Cu-eta".to_string())),
            k: parameters.get_spectrum("conductor.k", Some(&state.material_attributes))?
                .unwrap_or_else(|| Spectrum::Named("metal-Cu-k".to_string())),
            reflectance: parameters.get_spectrum("reflectance", Some(&state.material_attributes))?,
            coating: parse_coating(&state, &parameters)?,
        }),
        "conductor" => Ok(Material::Conductor {
            normal: parse_bump_normal_map(&state, &parameters)?,
            roughness: parse_roughness!(state, parameters, "")?,
            eta: parameters.get_texture_ref("eta", Some(&state.material_attributes))?
                .unwrap_or_else(|| TextureRef::Spectrum(Spectrum::Named("metal-Cu-eta".to_string()))),
            k: parameters.get_texture_ref("k", Some(&state.material_attributes))?
                .unwrap_or_else(|| TextureRef::Spectrum(Spectrum::Named("metal-Cu-k".to_string()))),
            reflectance: parameters.get_texture_ref("reflectance", Some(&state.material_attributes))?,
        }),
        "dielectric" => Ok(Material::Dielectric {
            normal: parse_bump_normal_map(&state, &parameters)?,
            roughness: parse_roughness!(state, parameters, "")?,
            eta: parameters.get_texture_ref("eta", Some(&state.material_attributes))?
                .unwrap_or(TextureRef::Float(1.5)),
        }),
        "diffuse" => Ok(Material::Diffuse {
            normal: parse_bump_normal_map(&state, &parameters)?,
            reflectance: parameters.get_texture_ref("reflectance", Some(&state.material_attributes))?
                .unwrap_or(TextureRef::Float(0.5)),
        }),
        "diffusetransmission" => Ok(Material::DiffuseTransmission {
            normal: parse_bump_normal_map(&state, &parameters)?,
            reflectance: parameters.get_texture_ref("reflectance", Some(&state.material_attributes))?
                .unwrap_or(TextureRef::Float(0.25)),
            transmittance: parameters.get_texture_ref("transmitttance", Some(&state.material_attributes))?
                .unwrap_or(TextureRef::Float(0.25)),
            scale: parameters.get_texture_ref("scale", Some(&state.material_attributes))?
                .unwrap_or(TextureRef::Float(1.0)),
        }),
        "hair" => Ok(Material::Hair {
            normal: parse_bump_normal_map(&state, &parameters)?,
            sigma_a: parameters.get_texture_ref("sigma_a", Some(&state.material_attributes))?,
            reflectance: parameters.get_texture_ref("reflectance", Some(&state.material_attributes))?,
            eumelanin: parameters.get_texture_ref("eumelanin", Some(&state.material_attributes))?,
            pheomelanin: parameters.get_texture_ref("pheomelanin", Some(&state.material_attributes))?,
            eta: parameters.get_texture_ref("eta", Some(&state.material_attributes))?
                .unwrap_or(TextureRef::Float(1.55)),
            beta_m: parameters.get_texture_ref("beta_m", Some(&state.material_attributes))?
                .unwrap_or(TextureRef::Float(0.3)),
            beta_n: parameters.get_texture_ref("beta_n", Some(&state.material_attributes))?
                .unwrap_or(TextureRef::Float(0.3)),
            alpha: parameters.get_texture_ref("alpha", Some(&state.material_attributes))?
                .unwrap_or(TextureRef::Float(2.0)),
        }),
        "interface" => Ok(Material::Interface),
        "measured" => Ok(Material::Measured {
            normal: parse_bump_normal_map(&state, &parameters)?,
            filename: parameters.get_string("filename", Some(&state.material_attributes))?
                .map(|s| s.clone()).ok_or("missing measured material filename")?,
        }),
        "mix" => Ok(Material::Mix {
            materials: parameters.get_strings("materials", Some(&state.material_attributes))?
                .filter(|v| v.len() == 2).map(|v| [v[0].clone(), v[1].clone()])
                .ok_or("missing mix materials")?,
            amount: parameters.get_texture_ref("amount", Some(&state.material_attributes))?
                .unwrap_or(TextureRef::Float(0.5)),
        }),
        "subsurface" => Ok(Material::Subsurface {
            normal: parse_bump_normal_map(&state, &parameters)?,
            roughness: parse_roughness!(state, parameters, "")?,
            eta: parameters.get_texture_ref("eta", Some(&state.material_attributes))?
                .unwrap_or(TextureRef::Float(1.33)),
            asymmetry: parameters.get_texture_ref("g", Some(&state.material_attributes))?
                .unwrap_or(TextureRef::Float(0.0)),
            mfp: parameters.get_texture_ref("mfp", Some(&state.material_attributes))?,
            name: parameters.get_string("name", Some(&state.material_attributes))?
                .map(|s| s.clone()),
            reflectance: parameters.get_texture_ref("reflectance", Some(&state.material_attributes))?,
            sigma_a: parameters.get_texture_ref("sigma_a", Some(&state.material_attributes))?
                .unwrap_or(TextureRef::RGB(glam::vec3(0.0011, 0.0024, 0.014))),
            sigma_s: parameters.get_texture_ref("sigma_s", Some(&state.material_attributes))?
                .unwrap_or(TextureRef::RGB(glam::vec3(2.55, 3.12, 3.77))),
            scale: parameters.get_float("scale", Some(&state.material_attributes))?
                .unwrap_or(1.0),
        }),
        _ => Err("invalid material")
    }
}

fn parse_texture_encoding(s: Option<&String>)
    -> Option<Result<TextureEncoding, &'static str>> {
    s.map(|s| match s.as_str() {
        "sRGB" => Ok(TextureEncoding::Srgb),
        "linear" => Ok(TextureEncoding::Linear),
        s if s.starts_with("gamma ") =>
            Ok(TextureEncoding::Gamma(s
                .strip_prefix("gamma ").unwrap().parse::<f32>()
                .map_err(|_| "failed to parse float")?)),
        _ => Err("invalid texture encoding")
    })
}

fn color_space_illuminant(color_space: &ColorSpace) -> Spectrum {
    match color_space {
        ColorSpace::Aces2065_1 => Spectrum::Named(String::from("illum-acesD60")),
        ColorSpace::Srgb | ColorSpace::Rec2020 | ColorSpace::DciP3 => Spectrum::Named(String::from("stdillum-D65")),
    }
}

pub fn parse_directive(state: &ParseState, tokenizer: &mut Tokenizer)
    -> Result<Option<Directive>, String> {
    let t = match tokenizer.next_token()? {
        Some(t) => t,
        None => return Ok(None),
    };

    match t.token.as_str() {
        "Identity" => Ok(Some(Directive::Identity)),
        "Translate" => Ok(Some(Directive::Translate(expect_vec3(tokenizer)?))),
        "Scale" => Ok(Some(Directive::Scale(expect_vec3(tokenizer)?))),
        "Rotate" => Ok(Some(Directive::Rotate {
            angle: expect_float(tokenizer)?,
            axis: expect_vec3(tokenizer)?,
        })),
        "LookAt" => Ok(Some(Directive::LookAt {
            eye: expect_vec3(tokenizer)?,
            look: expect_vec3(tokenizer)?,
            up: expect_vec3(tokenizer)?,
        })),
        "CoordinateSystem" => Ok(Some(Directive::CoordinateSystem {
            name: expect_string(tokenizer)?,
        })),
        "CoordSysTransform" => Ok(Some(Directive::CoordSysTransform {
            name: expect_string(tokenizer)?,
        })),
        "Transform" => Ok(Some(Directive::Transform(expect_mat4(tokenizer)?))),
        "ConcatTransform" => Ok(Some(Directive::ConcatTransform(expect_mat4(tokenizer)?))),
        "TransformTimes" => Ok(Some(Directive::TransformTimes {
            start: expect_float(tokenizer)?,
            end: expect_float(tokenizer)?,
        })),
        "ActiveTransform" => Ok(Some(Directive::ActiveTransform(
            match tokenizer.expect_token()?.token.as_str() {
                "StartTime" => Ok(ActiveTransform::StartTime),
                "EndTime" => Ok(ActiveTransform::EndTime),
                "All" => Ok(ActiveTransform::All),
                _ => Err("invalid active transform"),
            }?
        ))),
        "Include" => Ok(Some(Directive::Include(expect_string(tokenizer)?))),
        "Import" => Ok(Some(Directive::Import(expect_string(tokenizer)?))),
        "Option" => Ok(Some(Directive::Option(
            parse_named_parameter(tokenizer)?.ok_or("expected parameter")?))),
        "Camera" => {
            let camera_type = expect_string(tokenizer)?;
            let parameters = parse_parameter_list(tokenizer)?;

            Ok(Some(Directive::Camera {
                camera: match camera_type.as_str() {
                    "orthographic" => CameraType::Orthographic {
                        frame_aspect_ratio: parameters.get_float("frameaspectratio", None)?,
                        screen_window: parameters.get_float("screenwindow", None)?,
                        lens_radius: parameters.get_float("lensradius", None)?.unwrap_or(0.0),
                        focal_distance: parameters.get_float("focaldistance", None)?.unwrap_or(1.0e30),
                    },
                    "perspective" => CameraType::Perspective {
                        frame_aspect_ratio: parameters.get_float("frameaspectratio", None)?,
                        screen_window: parameters.get_float("screenwindow", None)?,
                        lens_radius: parameters.get_float("lensradius", None)?.unwrap_or(0.0),
                        focal_distance: parameters.get_float("focaldistance", None)?.unwrap_or(1.0e30),
                        fov: parameters.get_float("fov", None)?.unwrap_or(90.0),
                    },
                    "spherical" => CameraType::Spherical {
                        mapping: parameters.get_string("mapping", None)?.map(
                            |mapping| match mapping.as_str() {
                                "equirectangular" => Ok(SphericalMapping::Equirectangular),
                                "equalarea" => Ok(SphericalMapping::Equalarea),
                                _ => Err("invalid spherical camera mapping"),
                            }
                        ).unwrap_or(Ok(SphericalMapping::Equalarea))?
                    },
                    "realistic" => CameraType::Realistic {
                        lens_file: parameters.get_string("lensfile", None)?
                            .map(|s| s.clone()).ok_or("missing lens file")?,
                        aperture_diameter: parameters.get_float("aperturediameter", None)?.unwrap_or(1.0),
                        focus_distance: parameters.get_float("focus_distance", None)?.unwrap_or(10.0),
                        aperture: parameters.get_string("aperture", None)?.map(
                            |aperture| match aperture.as_str() {
                                "circular" => ApertureShape::Circular,
                                "gaussian" => ApertureShape::Gaussian,
                                "square" => ApertureShape::Square,
                                "pentagon" => ApertureShape::Pentagon,
                                "star" => ApertureShape::Star,
                                _ => ApertureShape::Custom(aperture.clone())
                            }
                        ).unwrap_or(ApertureShape::Circular),
                    },
                    _ => return Err(format!("invalid camera type: {}", camera_type)),
                },
                shutter_open: parameters.get_float("shutteropen", None)?.unwrap_or(0.0),
                shutter_close: parameters.get_float("shutterclose", None)?.unwrap_or(1.0),
            }))
        },
        "Sampler" => {
            let sampler = expect_string(tokenizer)?;
            let parameters = parse_parameter_list(tokenizer)?;

            Ok(Some(Directive::Sampler {
                sampler: match sampler.as_str() {
                    "halton" => SamplerType::Halton,
                    "independent" => SamplerType::Independent,
                    "paddedsobol" => SamplerType::PaddedSobol,
                    "sobol" => SamplerType::Sobol,
                    "stratified" => SamplerType::Stratified,
                    "zsobol" => SamplerType::ZSobol,
                    _ => return Err(format!("invalid sampler: {}",sampler)),
                },
                seed: parameters.get_integer("seed", None)?.unwrap_or(0)
            }))
        }
        "ColorSpace" => Ok(Some(Directive::ColorSpace(
            match expect_string(tokenizer)?.as_str() {
                "aces2065-1" => ColorSpace::Aces2065_1,
                "rec2020" => ColorSpace::Rec2020,
                "dci-p3" => ColorSpace::DciP3,
                "srgb" => ColorSpace::Srgb,
                color_space => return Err(format!("invalid color space: {}", color_space)),
            }
        ))),
        "Film" => {
            let _ = expect_string(tokenizer)?;
            let parameters = parse_parameter_list(tokenizer)?;

            let x_resolution = parameters.get_integer("xresolution", None)?.unwrap_or(1280);
            let y_resolution = parameters.get_integer("yresolution", None)?.unwrap_or(720);

            Ok(Some(Directive::Film {
                x_resolution,
                y_resolution,
                crop_window: parameters.get_floats("cropwindow", None)?
                    .filter(|f| f.len() == 4)
                    .map(|f| [glam::vec2(f[0], f[2]), glam::vec2(f[1], f[3])])
                    .unwrap_or([glam::vec2(0.0, 0.0), glam::vec2(1.0, 1.0)]),
                pixel_bounds: parameters.get_integers("pixelbounds", None)?
                    .filter(|f| f.len() == 4)
                    .map(|f| [glam::ivec2(f[0], f[2]), glam::ivec2(f[1], f[3])])
                    .unwrap_or([glam::ivec2(0, 0), glam::ivec2(x_resolution, y_resolution)]),
                diagonal: parameters.get_float("diagonal", None)?.unwrap_or(35.0),
                filename: parameters.get_string("filename", None)?
                    .map(|s| s.clone()).unwrap_or_else(|| String::from("output.exr")),
                iso: parameters.get_float("iso", None)?.unwrap_or(100.0),
                white_balance: parameters.get_float("whitebalance", None)?.unwrap_or(0.0),
                sensor: parameters.get_string("sensor", None)?
                    .map(|s| s.clone()).unwrap_or_else(|| String::from("cie1931"))
            }))
        },
        "PixelFilter" => {
            let filter = expect_string(tokenizer)?;
            let parameters = parse_parameter_list(tokenizer)?;

            Ok(Some(Directive::PixelFilter {
                filter: match filter.as_str() {
                    "box" => PixelFilter::Box {
                        x_radius: parameters.get_float("xradius", None)?.unwrap_or(0.5),
                        y_radius: parameters.get_float("yradius", None)?.unwrap_or(0.5),
                    },
                    "gaussian" => PixelFilter::Gaussian {
                        x_radius: parameters.get_float("xradius", None)?.unwrap_or(1.5),
                        y_radius: parameters.get_float("yradius", None)?.unwrap_or(1.5),
                        sigma: parameters.get_float("sigma", None)?.unwrap_or(0.5),
                    },
                    "mitchell" => PixelFilter::Mitchell {
                        x_radius: parameters.get_float("xradius", None)?.unwrap_or(2.0),
                        y_radius: parameters.get_float("yradius", None)?.unwrap_or(2.0),
                        b: parameters.get_float("B", None)?.unwrap_or(1.0 / 3.0),
                        c: parameters.get_float("B", None)?.unwrap_or(1.0 / 3.0),
                    },
                    "sinc" => PixelFilter::LanczosSinc {
                        x_radius: parameters.get_float("xradius", None)?.unwrap_or(4.0),
                        y_radius: parameters.get_float("yradius", None)?.unwrap_or(4.0),
                        tau: parameters.get_float("tau", None)?.unwrap_or(3.0),
                    },
                    "triangle" => PixelFilter::Triangle {
                        x_radius: parameters.get_float("xradius", None)?.unwrap_or(2.0),
                        y_radius: parameters.get_float("yradius", None)?.unwrap_or(2.0),
                    },
                    _ => return Err(format!("invalid pixel filter: {}", filter))
                }
            }))
        },
        "Integrator" => {
            let _ = expect_string(tokenizer)?;
            let _ = parse_parameter_list(tokenizer)?;

            Ok(Some(Directive::Unimplemented("Integrator")))
        },
        "Accelerator" => {
            let _ = expect_string(tokenizer)?;
            let _ = parse_parameter_list(tokenizer)?;

            Ok(Some(Directive::Unimplemented("Accelerator")))
        },
        "WorldBegin" => Ok(Some(Directive::WorldBegin)),
        "AttributeBegin" => Ok(Some(Directive::AttributeBegin)),
        "AttributeEnd" => Ok(Some(Directive::AttributeEnd)),
        "ReverseOrientation" => Ok(Some(Directive::ReverseOrientation)),
        "Attribute" => {
            let target = expect_string(tokenizer)?;
            let parameters = parse_parameter_list(tokenizer)?;

            Ok(Some(Directive::Attribute {
                target: match target.as_str() {
                    "shape" => AttributeTarget::Shape,
                    "light" => AttributeTarget::Light,
                    "material" => AttributeTarget::Material,
                    "medium" => AttributeTarget::Medium,
                    "texture" => AttributeTarget::Texture,
                    _ => return Err(format!("invalid attribute target: {}", target)),
                },
                parameters
            }))
        },
        "Shape" => {
            let name = expect_string(tokenizer)?;
            let parameters = parse_parameter_list(tokenizer)?;

            Ok(Some(Directive::Shape {
                shape: match name.as_str() {
                    "curve" => Shape::Curve {
                        points: parameters.get_points3("P", Some(&state.shape_attributes))?
                            .filter(|v| v.len() == 4)
                            .map(|v| [v[0], v[1], v[2], v[3]]),
                        basis: parameters.get_string("basis", Some(&state.shape_attributes))?
                            .map(|s| match s.as_str() {
                                "bezier" => Ok(CurveBasis::Bezier),
                                "bspline" => Ok(CurveBasis::BSpline),
                                _ => Err("invalid curve basis"),
                            }).unwrap_or(Ok(CurveBasis::Bezier))?,
                        degree: parameters.get_integer("degree", Some(&state.shape_attributes))?
                            .unwrap_or(3),
                        variant: parameters.get_string("type", Some(&state.shape_attributes))?
                            .map(|s| match s.as_str() {
                                "flat" => Ok(CurveVariant::Flat),
                                "cylinder" => Ok(CurveVariant::Cylinder),
                                "ribbon" => Ok(CurveVariant::Ribbon),
                                _ => Err("invalid curve variant"),
                            }).unwrap_or(Ok(CurveVariant::Flat))?,
                        normals: parameters.get_normals3("N", Some(&state.shape_attributes))?
                            .filter(|v| v.len() == 2)
                            .map(|v| [v[0], v[1]]),
                        start_width: parameters.get_float("width0", Some(&state.shape_attributes))?
                            .map(|f| Ok::<f32, String>(f))
                            .unwrap_or_else(|| parameters.get_float("width", Some(&state.shape_attributes))?
                                .map(|f| Ok(f)).unwrap_or(Ok(1.0)))?,
                        end_width: parameters.get_float("width1", Some(&state.shape_attributes))?
                            .map(|f| Ok::<f32, String>(f))
                            .unwrap_or_else(|| parameters.get_float("width", Some(&state.shape_attributes))?
                                .map(|f| Ok(f)).unwrap_or(Ok(1.0)))?,
                        split_depth: parameters.get_integer("splitdepth", Some(&state.shape_attributes))?
                            .unwrap_or(3)
                    },
                    "cylinder" => Shape::Cylinder {
                        radius: parameters.get_float("radius", Some(&state.shape_attributes))?.unwrap_or(1.0),
                        z_min: parameters.get_float("zmin", Some(&state.shape_attributes))?.unwrap_or(-1.0),
                        z_max: parameters.get_float("zmax", Some(&state.shape_attributes))?.unwrap_or(1.0),
                        phi_max: parameters.get_float("phimax", Some(&state.shape_attributes))?.unwrap_or(360.0),
                    },
                    "disk" => Shape::Disk {
                        height: parameters.get_float("height", Some(&state.shape_attributes))?.unwrap_or(0.0),
                        radius: parameters.get_float("radius", Some(&state.shape_attributes))?.unwrap_or(1.0),
                        inner_radius: parameters.get_float("innerradius", Some(&state.shape_attributes))?.unwrap_or(360.0),
                        phi_max: parameters.get_float("phimax", Some(&state.shape_attributes))?.unwrap_or(360.0),
                    },
                    "sphere" => {
                        let radius = parameters.get_float("radius", Some(&state.shape_attributes))?.unwrap_or(1.0);

                        Shape::Sphere {
                            radius,
                            z_min: parameters.get_float("zmin", Some(&state.shape_attributes))?.unwrap_or(-radius),
                            z_max: parameters.get_float("zmax", Some(&state.shape_attributes))?.unwrap_or(radius),
                            phi_max: parameters.get_float("phimax", Some(&state.shape_attributes))?.unwrap_or(360.0),
                        }
                    },
                    "trianglemesh" => Shape::TriangleMesh {
                        indices: parameters.get_integers("indices", Some(&state.shape_attributes))?
                            .map(|v| v.clone()),
                        vertices: parameters.get_points3("P", Some(&state.shape_attributes))?
                            .ok_or("missing vertex positions")?.clone(),
                        normals: parameters.get_normals3("N", Some(&state.shape_attributes))?
                            .map(|v| v.clone()),
                        tangents: parameters.get_vectors3("S", Some(&state.shape_attributes))?
                            .map(|v| v.clone()),
                        uvs: parameters.get_points2("uv", Some(&state.shape_attributes))?
                            .map(|v| v.clone()),
                    },
                    "plymesh" => Shape::PlyMesh {
                        filename: parameters.get_string("filename", Some(&state.shape_attributes))?
                            .ok_or("missing mesh filename")?.clone(),
                        displacement: parameters.get_texture_ref("displacement", Some(&state.shape_attributes))?,
                        edge_length: parameters.get_float("edgelength", Some(&state.shape_attributes))?.unwrap_or(1.0),
                    },
                    _ => return Err(format!("invalid shape type: {}", name))
                },
                alpha: parameters.get_texture_ref("alpha", Some(&state.shape_attributes))?
                    .unwrap_or(TextureRef::Float(1.0)),
            }))
        }
        "ObjectBegin" => Ok(Some(Directive::ObjectBegin {
            name: expect_string(tokenizer)?,
        })),
        "ObjectEnd" => Ok(Some(Directive::ObjectEnd)),
        "ObjectInstance" => Ok(Some(Directive::ObjectInstance {
            name: expect_string(tokenizer)?,
        })),
        "LightSource" => {
            let source_type = expect_string(tokenizer)?;
            let parameters = parse_parameter_list(tokenizer)?;

            Ok(Some(Directive::LightSource {
                light: match source_type.as_str() {
                    "distant" => Light::Distant {
                        illuminant: parameters.get_spectrum("L", Some(&state.light_attributes))?
                            .unwrap_or_else(|| color_space_illuminant(&state.color_space)),
                        from: parameters.get_point3("from", Some(&state.light_attributes))?
                            .unwrap_or(glam::vec3(0.0, 0.0, 0.0)),
                        to: parameters.get_point3("to", Some(&state.light_attributes))?
                            .unwrap_or(glam::vec3(0.0, 0.0, 1.0)),
                    },
                    "goniometric" => Light::Goniometric {
                        filename: parameters.get_string("filename", Some(&state.light_attributes))?
                            .map(|s| s.clone()).ok_or("missing image filename")?,
                        illuminant: parameters.get_spectrum("I", Some(&state.light_attributes))?
                            .unwrap_or_else(|| color_space_illuminant(&state.color_space)),
                    },
                    "infinite" => Light::Infinite {
                        filename: parameters.get_string("filename", Some(&state.light_attributes))?
                            .map(|s| s.clone()),
                        portal: parameters.get_points3("portal", Some(&state.light_attributes))?
                            .filter(|v| v.len() == 4).map(|v| [v[0], v[1], v[2], v[3]]),
                        illuminant: parameters.get_spectrum("L", Some(&state.light_attributes))?
                            .unwrap_or_else(|| color_space_illuminant(&state.color_space)),
                    },
                    "point" => Light::Point {
                        illuminant: parameters.get_spectrum("I", Some(&state.light_attributes))?
                            .unwrap_or_else(|| color_space_illuminant(&state.color_space)),
                        from: parameters.get_point3("from", Some(&state.light_attributes))?
                            .unwrap_or(glam::vec3(0.0, 0.0, 0.0)),
                    },
                    "projection" => Light::Projection {
                        illuminant: parameters.get_spectrum("I", Some(&state.light_attributes))?
                            .unwrap_or_else(|| color_space_illuminant(&state.color_space)),
                        fov: parameters.get_float("fov", Some(&state.light_attributes))?
                            .unwrap_or(90.0),
                        filename: parameters.get_string("filename", Some(&state.light_attributes))?
                            .map(|s| s.clone()).ok_or("missing image filename")?,
                    },
                    "spot" => Light::Spotlight {
                        illuminant: parameters.get_spectrum("I", Some(&state.light_attributes))?
                            .unwrap_or_else(|| color_space_illuminant(&state.color_space)),
                        from: parameters.get_point3("from", Some(&state.light_attributes))?
                            .unwrap_or(glam::vec3(0.0, 0.0, 0.0)),
                        to: parameters.get_point3("to", Some(&state.light_attributes))?
                            .unwrap_or(glam::vec3(0.0, 0.0, 1.0)),
                        cone_angle: parameters.get_float("coneangle", Some(&state.light_attributes))?
                            .unwrap_or(30.0),
                        cone_delta_angle: parameters.get_float("conedeltaangle", Some(&state.light_attributes))?
                            .unwrap_or(5.0),
                    },
                    _ => return Err(format!("invalid light source type: {}", source_type)),
                },
                illuminance: parameters.get_float("power", Some(&state.light_attributes))?
                    .or_else(|| parameters.get_float("illuminance", Some(&state.light_attributes)).unwrap_or(None)),
                scale: parameters.get_float("scale", Some(&state.light_attributes))?.unwrap_or(1.0),
            }))
        },
        "AreaLightSource" => {
            let light_type = expect_string(tokenizer)?;
            let parameters = parse_parameter_list(tokenizer)?;

            if light_type.as_str() != "diffuse" {
                return Err(format!("invalid area light source type: {}", light_type))
            }

            Ok(Some(Directive::AreaLightSource(AreaLightSource {
                filename: parameters.get_string("filename", Some(&state.light_attributes))?
                    .map(|s| s.clone()),
                illuminant: parameters.get_spectrum("L", Some(&state.light_attributes))?
                    .unwrap_or_else(|| color_space_illuminant(&state.color_space)),
                two_sided: parameters.get_bool("twosided", Some(&state.light_attributes))?.unwrap_or(false),
            })))
        },
        "Material" => Ok(Some(Directive::Material(parse_material(state, tokenizer)?))),
        "MakeNamedMaterial" => Ok(Some(Directive::MakeNamedMaterial {
            name: expect_string(tokenizer)?,
            material: parse_material(state, tokenizer)?,
        })),
        "NamedMaterial" => Ok(Some(Directive::NamedMaterial {
            name: expect_string(tokenizer)?,
        })),
        "Texture" => {
            let name = expect_string(tokenizer)?;
            let texture_type = match expect_string(tokenizer)?.as_str() {
                "spectrum" => TextureType::Spectrum,
                "float" => TextureType::Float,
                t => return Err(format!("invalid texture type: {}", t)),
            };

            let class = expect_string(tokenizer)?;
            let parameters = parse_parameter_list(tokenizer)?;

            let scale = glam::vec2(
                parameters.get_float("uscale", Some(&state.texture_attributes))?.unwrap_or(1.0),
                parameters.get_float("vscale", Some(&state.texture_attributes))?.unwrap_or(1.0),
            );
            let delta = glam::vec2(
                parameters.get_float("udelta", Some(&state.texture_attributes))?.unwrap_or(0.0),
                parameters.get_float("vdelta", Some(&state.texture_attributes))?.unwrap_or(0.0),
            );

            let mapping = parameters.get_string("mapping", Some(&state.texture_attributes))?
                .map(|s| match s.as_str() {
                    "uv" => Ok(TextureMapping::Uv { scale, delta }),
                    "spherical" => Ok(TextureMapping::Spherical),
                    "cylindrical" => Ok(TextureMapping::Cylindrical),
                    "planar" => Ok(TextureMapping::Planar {
                        delta,
                        v1: parameters.get_vector3("v1", Some(&state.texture_attributes))?.unwrap_or(glam::vec3(1.0, 0.0, 0.0)),
                        v2: parameters.get_vector3("v1", Some(&state.texture_attributes))?.unwrap_or(glam::vec3(0.0, 1.0, 0.0)),
                    }),
                    _ => Err("invalid texture mapping"),
                }).unwrap_or(Ok(TextureMapping::Uv { scale, delta }))?;

            Ok(Some(Directive::Texture {
                name, texture_type, mapping,
                texture: match class.as_str() {
                    "bilerp" => Texture::BilinearInterpolation {
                        v00: parameters.get_texture_ref("v00", Some(&state.texture_attributes))?.unwrap_or(TextureRef::Float(0.0)),
                        v01: parameters.get_texture_ref("v01", Some(&state.texture_attributes))?.unwrap_or(TextureRef::Float(1.0)),
                        v10: parameters.get_texture_ref("v10", Some(&state.texture_attributes))?.unwrap_or(TextureRef::Float(0.0)),
                        v11: parameters.get_texture_ref("v11", Some(&state.texture_attributes))?.unwrap_or(TextureRef::Float(1.0)),
                    },
                    "checkerboard" => Texture::Checkerboard {
                        dimension: parameters.get_integer("dimension", Some(&state.texture_attributes))?.unwrap_or(2),
                        texture1: parameters.get_texture_ref("tex1", Some(&state.texture_attributes))?.unwrap_or(TextureRef::Float(1.0)),
                        texture2: parameters.get_texture_ref("tex2", Some(&state.texture_attributes))?.unwrap_or(TextureRef::Float(0.0)),
                    },
                    "constant" => Texture::Constant {
                        value: parameters.get_texture_ref("value", Some(&state.texture_attributes))?.unwrap_or(TextureRef::Float(1.0)),
                    },
                    "directionmix" => Texture::DirectionMix {
                        texture1: parameters.get_texture_ref("tex1", Some(&state.texture_attributes))?.unwrap_or(TextureRef::Float(0.0)),
                        texture2: parameters.get_texture_ref("tex1", Some(&state.texture_attributes))?.unwrap_or(TextureRef::Float(1.0)),
                        direction: parameters.get_vector3("dir", Some(&state.texture_attributes))?.unwrap_or(glam::vec3(0.0, 1.0, 0.0)),
                    },
                    "dots" => Texture::Dots {
                        inside: parameters.get_texture_ref("inside", Some(&state.texture_attributes))?.unwrap_or(TextureRef::Float(1.0)),
                        outside: parameters.get_texture_ref("outside", Some(&state.texture_attributes))?.unwrap_or(TextureRef::Float(0.0)),
                    },
                    "fbm" => Texture::Fbm {
                        octaves: parameters.get_integer("octaves", Some(&state.texture_attributes))?.unwrap_or(8),
                        roughness: parameters.get_float("roughness", Some(&state.texture_attributes))?.unwrap_or(0.5),
                    },
                    "wrinkled" => Texture::Wrinkled {
                        octaves: parameters.get_integer("octaves", Some(&state.texture_attributes))?.unwrap_or(8),
                        roughness: parameters.get_float("roughness", Some(&state.texture_attributes))?.unwrap_or(0.5),
                    },
                    "windy" => Texture::Windy {
                        octaves: parameters.get_integer("octaves", Some(&state.texture_attributes))?.unwrap_or(8),
                        roughness: parameters.get_float("roughness", Some(&state.texture_attributes))?.unwrap_or(0.5),
                    },
                    "imagemap" => Texture::ImageMap {
                        filename: parameters.get_string("filename", Some(&state.texture_attributes))?
                            .map(|s| s.clone()).ok_or("missing image filename")?,
                        wrap: parameters.get_string("wrap", Some(&state.texture_attributes))?
                            .map(|s| match s.as_str() {
                                "repeat" => Ok(TextureWrap::Repeat),
                                "black" => Ok(TextureWrap::Black),
                                "clamp" => Ok(TextureWrap::Clamp),
                                _ => Err("invalid texture wrap"),
                            }).unwrap_or(Ok(TextureWrap::Repeat))?,
                        max_anisotropy: parameters.get_float("maxanisotropy", Some(&state.texture_attributes))?.unwrap_or(8.0),
                        filter: parameters.get_string("filter", Some(&state.texture_attributes))?
                            .map(|s| match s.as_str() {
                                "bilinear" => Ok(FilterMode::Bilinear),
                                "ewa" => Ok(FilterMode::Ewa),
                                "trilinear" => Ok(FilterMode::Trilinear),
                                "point" => Ok(FilterMode::Point),
                                _ => Err("invalid filter mode"),
                            }).unwrap_or(Ok(FilterMode::Bilinear))?,
                        encoding: parse_texture_encoding(parameters
                            .get_string("encoding", Some(&state.texture_attributes))?)
                            .unwrap_or(Ok(TextureEncoding::Srgb))?,
                        scale: parameters.get_float("scale", Some(&state.texture_attributes))?.unwrap_or(1.0),
                        invert: parameters.get_bool("invert", Some(&state.texture_attributes))?.unwrap_or(false),
                    },
                    "marble" => Texture::Marble {
                        octaves: parameters.get_integer("octaves", Some(&state.texture_attributes))?.unwrap_or(8),
                        roughness: parameters.get_float("roughness", Some(&state.texture_attributes))?.unwrap_or(0.5),
                        scale: parameters.get_float("scale", Some(&state.texture_attributes))?.unwrap_or(1.0),
                        variation: parameters.get_float("variation", Some(&state.texture_attributes))?.unwrap_or(0.2),
                    },
                    "mix" => Texture::Mix {
                        texture1: parameters.get_texture_ref("tex1", Some(&state.texture_attributes))?.unwrap_or(TextureRef::Float(0.0)),
                        texture2: parameters.get_texture_ref("tex2", Some(&state.texture_attributes))?.unwrap_or(TextureRef::Float(1.0)),
                        amount: parameters.get_texture_ref("amount", Some(&state.texture_attributes))?.unwrap_or(TextureRef::Float(0.5)),
                    },
                    "ptex" => Texture::Ptex {
                        encoding: parse_texture_encoding(parameters
                            .get_string("encoding", Some(&state.texture_attributes))?)
                            .unwrap_or(Ok(TextureEncoding::Gamma(2.2)))?,
                        filename: parameters.get_string("filename", Some(&state.texture_attributes))?
                            .map(|s| s.clone()).ok_or("missing ptex filename")?,
                        scale: parameters.get_float("scale", Some(&state.texture_attributes))?.unwrap_or(1.0),
                    },
                    "scale" => Texture::Scale {
                        texture: parameters.get_texture_ref("tex", Some(&state.texture_attributes))?.unwrap_or(TextureRef::Float(1.0)),
                        scale: parameters.get_texture_ref("scale", Some(&state.texture_attributes))?.unwrap_or(TextureRef::Float(1.0)),
                    },
                    _ => return Err(format!("invalid texture type: {}", class)),
                }
            }))
        },
        "MediumInterface" => {
            let _ = expect_string(tokenizer)?;
            let _ = expect_string(tokenizer)?;

            Ok(Some(Directive::Unimplemented("MediumInterface")))
        },
        _ => Err(format!("unrecognized directive: {}", t.token)),
    }
}

#[derive(Debug, Clone, Default)]
pub struct Transformation {
    matrix: glam::Mat4,
}

#[derive(Debug, Clone, Default)]
pub struct ParseState {
    shape_attributes: ParameterDictionary,
    light_attributes: ParameterDictionary,
    material_attributes: ParameterDictionary,
    medium_attributes: ParameterDictionary,
    texture_attributes: ParameterDictionary,
    current_material: Material,
    color_space: ColorSpace,
    transformation: Transformation,
    area_light: Option<AreaLightSource>,
}