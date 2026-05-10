enable wgpu_ray_query;

const PI: f32 = 3.14159265358979323846;
const MISSING_UV_COORD: f32 = -1.0;

struct CameraUniform {
    view_inverse: mat4x4<f32>,
    projection_inverse: mat4x4<f32>,
    frame_index: u32,
}

struct VertexBuffer {
    position_u: vec4f,
    normal_v: vec4f,
}

struct PartialMaterial {
    material_flags: u32,
    displacement: i32,
    normal_map: i32,
    roughness_u: i32,
    roughness_v: i32,
    reflectance: i32,
    eta: i32,
    k: i32,
}

struct Material {
    material1: PartialMaterial,
    material2: PartialMaterial,
    mix_factor: i32,
}

struct InstanceData {
    first_index: u32,
    first_vertex: u32,
    material: Material,
}

const IMAGE_FLAG_INVERT: u32 = 1;

struct ImageInfo {
    uv_scale: vec2f,
    uv_delta: vec2f,
    scale: f32,
    gamma: f32,
    flags: u32,
}

const TEXTURE_CONSTANT: u32 = 0;
const TEXTURE_IMAGE: u32 = 1;
const TEXTURE_MIX: u32 = 2;
const TEXTURE_SCALE: u32 = 3;

struct Texture {
    tex_type: u32,
    tex0: u32,
    tex1: u32,
    tex2: u32,
}

struct TextureSamplingContext {
    uv: vec2f,
}

struct EnvironmentData {
    light_map: i32,
}

struct SpectralData {
    cie_cmf_xyz: array<vec3f, 441>,
    cie_bt709_basis: array<vec3f, 391>,
    illuminant_d65: array<f32, 97>,
    cie_y_integral: f32,
}

@group(0) @binding(0)
var output_texture: texture_storage_2d<rgba16float, write>;

@group(0) @binding(1)
var accumulation_buffer: texture_storage_2d<rgba32float, read_write>;

@group(0) @binding(2)
var<uniform> camera: CameraUniform;

@group(0) @binding(3)
var linear_sampler: sampler;

@group(0) @binding(4)
var<storage, read> spectral_data: SpectralData;

@group(0) @binding(5)
var<storage, read> solar_irradiance: array<f32>;

@group(1) @binding(0)
var scene: acceleration_structure;

@group(1) @binding(1)
var<storage, read> vertex_buffer: array<VertexBuffer>;

@group(1) @binding(2)
var<storage, read> index_buffer: array<u32>;

@group(1) @binding(3)
var<storage, read> instance_buffer: array<InstanceData>;

@group(1) @binding(4)
var image_array: binding_array<texture_2d<f32>>;

@group(1) @binding(5)
var<storage, read> image_info: array<ImageInfo>;

@group(1) @binding(6)
var<storage, read> texture_data: array<Texture>;

@group(1) @binding(7)
var<storage, read> environment_data: EnvironmentData;

struct RNGState {
    seed: vec3f,
}

var<private> rng_state: RNGState;

fn sampleTexture(index: u32, ctx: TextureSamplingContext) -> vec3f {
    let info = image_info[index];
    var data = textureSampleLevel(image_array[index], linear_sampler, vec2f(ctx.uv.x, 1.0 - ctx.uv.y) * info.uv_scale + info.uv_delta, 0).rgb;
    data = pow(data, vec3f(info.gamma));
    if ((info.flags & IMAGE_FLAG_INVERT) != 0) {
        data = 1.0 - data;
    }
    return data * info.scale;
}

fn resolveTextureNoMixScale(texture: Texture, ctx: TextureSamplingContext) -> vec3f {
    if (texture.tex_type == TEXTURE_CONSTANT) {
        return bitcast<vec3f>(vec3u(texture.tex0, texture.tex1, texture.tex2));
    } else if (texture.tex_type == TEXTURE_IMAGE) {
        return sampleTexture(texture.tex0, ctx);
    }

    return vec3f(0.0);
}

fn resolveTextureNoScale(texture: Texture, ctx: TextureSamplingContext) -> vec3f {
    if (texture.tex_type == TEXTURE_MIX) {
        let v0 = resolveTextureNoMixScale(texture_data[texture.tex0], ctx);
        let v1 = resolveTextureNoMixScale(texture_data[texture.tex1], ctx);
        let v2 = resolveTextureNoMixScale(texture_data[texture.tex2], ctx);
        return mix(v0, v1, v2);
    }

    return resolveTextureNoMixScale(texture, ctx);
}

fn resolveTextureNoMix(texture: Texture, ctx: TextureSamplingContext) -> vec3f {
    if (texture.tex_type == TEXTURE_SCALE) {
        let v0 = resolveTextureNoMixScale(texture_data[texture.tex0], ctx);
        let v1 = resolveTextureNoMixScale(texture_data[texture.tex1], ctx);
        return v0 * v1;
    }

    return resolveTextureNoMixScale(texture, ctx);
}

fn resolveTexture(texture: Texture, ctx: TextureSamplingContext) -> vec3f {
    if (texture.tex_type == TEXTURE_MIX) {
        let v0 = resolveTextureNoMix(texture_data[texture.tex0], ctx);
        let v1 = resolveTextureNoMix(texture_data[texture.tex1], ctx);
        let v2 = resolveTextureNoMix(texture_data[texture.tex2], ctx);
        return mix(v0, v1, v2);
    } else if (texture.tex_type == TEXTURE_SCALE) {
        let v0 = resolveTextureNoScale(texture_data[texture.tex0], ctx);
        let v1 = resolveTextureNoScale(texture_data[texture.tex1], ctx);
        return v0 * v1;
    }

    return resolveTextureNoMixScale(texture, ctx);
}

fn resolveTextureOr(texture: i32, ctx: TextureSamplingContext, fallback: vec3f) -> vec3f {
    if texture < 0 {
        return fallback;
    }

    return resolveTexture(texture_data[texture], ctx);
}

fn hash(x: u32) -> u32 {
    var r = x;
    r += (r << 10u);
    r ^= (r >> 6u);
    r += (r << 3u);
    r ^= (r >> 11u);
    r += (r << 15u);
    return r;
}

fn hash3(v: vec3u) -> u32 {
    return hash(v.x ^ hash(v.y) ^ hash(v.z));
}

fn floatConstruct(m: u32) -> f32 {
    const ieee_mantissa: u32 = 0x007FFFFFu;
    const ieee_one: u32 = 0x3F800000u;

    var r = m;
    r &= ieee_mantissa;
    r |= ieee_one;

    return fract(bitcast<f32>(r) - 1.0);
}

fn advanceRNG() {
    rng_state.seed += 1.0;
}

fn random1() -> f32 {
    advanceRNG();
    return floatConstruct(hash3(bitcast<vec3u>(rng_state.seed)));
}

fn random2() -> vec2f {
    return vec2f(random1(), random1());
}

fn random3() -> vec3f {
    return vec3f(random2(), random1());
}

fn random4() -> vec4f {
    return vec4f(random2(), random2());
}

fn buildOrthonormalBasis(normal: vec3f, b1: ptr<function, vec3f>, b2: ptr<function, vec3f>) {
    if (normal.z < -0.9999999) {
        *b1 = vec3f(0.0, -1.0, 0.0);
        *b2 = vec3f(-1.0, 0.0, 0.0);
    } else {
        let a = 1.0 / (1.0 + normal.z);
        let b = -normal.x * normal.y * a;
        *b1 = vec3f(1.0 - normal.x * normal.x * a, b, -normal.x);
        *b2 = vec3f(b, 1.0 - normal.y * normal.y * a, -normal.y);
    }
}

fn sampleDisk(rand: vec2f) -> vec2f {
    let angle = rand.y * 2.0 * PI;
    let sr = sqrt(rand.x);
    return vec2f(sr * cos(angle), sr * sin(angle));
}

fn sampleSphere(rand: vec2f) -> vec3f {
    let angle = rand.x * 2.0 * PI;
    let u = rand.y * 2.0 - 1.0;
    let sr = sqrt(1.0 - u * u);
    return vec3f(sr * cos(angle), sr * sin(angle), u);
}

fn sampleHemisphere(rand: vec2f) -> vec3f {
    var p = sampleSphere(rand);
    p.y = abs(p.y);
    return p;
}

fn sampleHemisphereAround(rand: vec2f, normal: vec3f) -> vec3f {
    let p = sampleSphere(rand);
    return p * sign(dot(p, normal));
}

fn sampleCosineHemisphere(rand: vec2f) -> vec3f {
    let p = sampleDisk(rand);
    return vec3f(p, sqrt(1.0 - dot(p, p)));
}

fn sampleCosineHemisphereAround(rand: vec2f, normal: vec3f) -> vec3f {
    let p = sampleCosineHemisphere(rand);

    var tangent: vec3f;
    var bitangent: vec3f;
    buildOrthonormalBasis(normal, &tangent, &bitangent);
    return tangent * p.x + bitangent * p.y + normal * p.z;
}

fn cosineWeightedHemispherePDF(direction: vec3f, normal: vec3f) -> f32 {
    return dot(direction, normal) / PI;
}

fn barycentricWeights(barycentrics: vec2f) -> vec3f {
    return vec3f(
        1.0 - barycentrics.x - barycentrics.y,
        barycentrics.x,
        barycentrics.y,
    );
}

fn fresnelRs(n0: f32, cos0: f32, n1: f32, cos1: f32) -> f32 {
    return (n0 * cos0 - n1 * cos1) / (n0 * cos0 + n1 * cos1);
}

fn fresnelRp(n0: f32, cos0: f32, n1: f32, cos1: f32) -> f32 {
    return (n0 * cos1 - n1 * cos0) / (n0 * cos1 + n1 * cos0);
}

fn fresnelDielectric(cosTheta0: f32, n0: f32, n1: f32) -> f32 {
    if (n0 == n1) {
        return 0.0;
    }

    let sin2Theta0 = 1.0 - cosTheta0 * cosTheta0;
    let sin2Theta1 = sin2Theta0 * n0 * n0 / (n1 * n1);
    if (sin2Theta1 >= 1.0) {
        return 1.0;
    }

    let cosTheta1 = sqrt(1.0 - sin2Theta1);

    let rs = fresnelRs(n0, cosTheta0, n1, cosTheta1);
    let rp = fresnelRp(n0, cosTheta0, n1, cosTheta1);

    return 0.5 * (rs * rs + rp * rp);
}

fn fresnelConductor(cosTheta: f32, N: vec3f, K: vec3f) -> vec3f {
    let cosTheta2 = cosTheta * cosTheta;
    let sinTheta2 = 1.0 - cosTheta2;
    let n2 = N * N;
    let k2 = K * K;

    let t0 = n2 - k2 - sinTheta2;
    let a2b2 = sqrt(t0 * t0 + 4.0 * n2 * k2);
    let t1 = a2b2 + cosTheta2;
    let a = sqrt(0.5 * (a2b2 + t0));
    let t2 = 2.0 * a * cosTheta;
    let Rs = (t1 - t2) / (t1 + t2);

    let t3 = cosTheta2 * a2b2 + sinTheta2 * sinTheta2;
    let t4 = t2 * sinTheta2;
    let Rp = Rs * (t3 - t4) / (t3 + t4);

    return 0.5 * (Rp + Rs);
}

fn sampleVndf(u: vec2f, wi: vec3f, alpha: f32, n: vec3f) -> vec3f {
    let wi_z = -n * dot(wi, n);
    let wi_xy = wi + wi_z;

    let wiStd = -normalize(alpha * wi_xy + wi_z);

    let wiStd_z = dot(wiStd, n);
    let z = 1.0 - u.y * (1.0 + wiStd_z);
    let sinTheta = sqrt(saturate(1.0f - z * z));
    let phi = 2.0 * PI * u.x - PI;
    let x = sinTheta * cos(phi);
    let y = sinTheta * sin(phi);
    let cStd = vec3f(x, y, z);

    let up = vec3f(0, 0, 1.000001);
    let wr = n + up;
    let c = dot(wr, cStd) * wr / wr.z - cStd;

    let wmStd = c + wiStd;
    let wmStd_z = n * dot(n, wmStd);
    let wmStd_xy = wmStd_z - wmStd;

    return normalize(alpha * wmStd_xy + wmStd_z);
}

fn octahedral_wrap(v: vec2f) -> vec2f {
    return (1.0 - abs(v.yx)) * (vec2f(v.xy >= vec2f(0.0)) * 2.0 - 1.0);
}

fn octahedral_encode(n: vec3f) -> vec2f {
    var N = n / (abs(n.x) + abs(n.y) + abs(n.z));
    var r = N.xz;
    if (N.y < 0.0) {
        r = octahedral_wrap(N.xz);
    }
    return r * 0.5 + 0.5;
}

const XYZ_TO_sRGB: mat3x3<f32> = mat3x3<f32>(
     3.2404542, -0.9692660,  0.0556434,
    -1.5371385,  1.8760108, -0.2040259,
    -0.4985314,  0.0415560,  1.0572252
);

const ACESInputMat: mat3x3<f32> = mat3x3<f32>(
    0.59719, 0.35458, 0.04823,
    0.07600, 0.90834, 0.01566,
    0.02840, 0.13383, 0.83777
);

// ODT_SAT => XYZ => D60_2_D65 => sRGB
const ACESOutputMat: mat3x3<f32> = mat3x3<f32>(
     1.60475, -0.53108, -0.07367,
    -0.10208,  1.10813, -0.00605,
    -0.00327, -0.07276,  1.07602
);

fn RRTAndODTFit(v: vec3f) -> vec3f {
    let a = v * (v + 0.0245786f) - 0.000090537f;
    let b = v * (0.983729f * v + 0.4329510f) + 0.238081f;
    return a / b;
}

fn acesFitted(color: vec3f) -> vec3f {
    return saturate(RRTAndODTFit(color * ACESInputMat) * ACESOutputMat);
}

struct ray {
    origin: vec3f,
    direction: vec3f,
}

fn CIE_Y_Integral() -> f32 {
    return spectral_data.cie_y_integral;
}

fn CIE_CMF_XYZ(lambda: f32) -> vec3f {
    if (lambda < 390.0 || lambda >= 831.0) {
        return vec3f(0.0);
    }
    return spectral_data.cie_cmf_xyz[i32(lambda) - 390];
}

fn CIE_BT709_Basis(lambda: f32) -> vec3f {
    if (lambda < 390.0 || lambda >= 781.0) {
        return vec3f(0.0);
    }
    return spectral_data.cie_bt709_basis[i32(lambda) - 390];
}

fn Illuminant_D65(lambda: f32) -> f32 {
    if (lambda < 300.0 || lambda >= 780.0) {
        return 0.0;
    }

    let index = (i32(lambda) - 300) / 5;
    let t = f32((i32(lambda) - 300) - index * 5) / 5.0;
    return mix(spectral_data.illuminant_d65[index], spectral_data.illuminant_d65[index + 1], t);
}

fn getSunRadiance(lambda: f32) -> f32 {
    return solar_irradiance[clamp(i32(lambda), 390, 830) - 390] * 1000.0 / (2.0 * PI);
}

fn sampleWavelength(u: f32, pdf: ptr<function, f32>) -> f32 {
    let wl = 556.638293609 - 130.023639424 * atanh(0.85691062 - 1.82750197 * u);
    let denom = cosh(4.28105 - 0.00769091 * wl);
    *pdf = 0.00420843 / (denom * denom);
    return clamp(wl, 390.0, 830.0);
}

const earthRadius: f32 = 6371.3e3;
const atmosphereRadius: f32 = 6451.3e3;

const astronomicalUnit: f32 = 149597870700.0;
const sunRadius: f32 = 695700.0e3;

const atmosphereTurbidity: f32 = 1.5;

const aerosolDiameter: f32 = 1.0;

fn sampleSunDirection(rand: vec2f, sunPosition: vec3f, x: vec3f, weight: ptr<function, f32>) -> vec3f {
    let centerDirection = normalize(sunPosition - x);

    var b1: vec3f;
    var b2: vec3f;
    buildOrthonormalBasis(centerDirection, &b1, &b2);

    let pLocal = sunRadius * sampleDisk(rand);
    let sunDiskPoint = pLocal.x * b1 + pLocal.y * b2 + sunPosition;
    let pDistance = distance(sunDiskPoint, x);

    let sampleDirection = (sunDiskPoint - x) / pDistance;

    let diskArea = sunRadius * sunRadius * PI;
    *weight = diskArea * dot(sampleDirection, centerDirection) / (pDistance * pDistance);

    return sampleDirection;
}

fn airDensity(x: f32) -> f32 {
    return 2.50844 / (1.0 + exp(0.000159087 * x + 0.0691795));
}

fn aerosolDensity(x: f32) -> f32 {
    if (x < 0.0) {
        return 0.0;
    }

    var c = 0.0;
    if (x < 1855.0) {
        c = ((((-1.37232e-9) * x + 0.00000399595) * x - 0.00362987) * x - 0.0895921) * x + 10027.7162;
    } else if (x < 2665.0) {
        c = 1.07158e14 * pow(x, -3.12345);
    } else if (x < 10275.0) {
        c = 2935.81129 * pow(0.999881, x);
    } else {
        c = 892.89874 / (1.0 + exp(-(-0.0019755 * x + 23.706)));
    }
    return 1.0e-3 * c * 1.0e-4;
}

fn ozoneDensity(x: f32) -> f32 {
    if (x < 0.0 || x > 74000.0) {
        return 0.0;
    }

    return exp((((-1.2272e-18 *
        x + 2.6322e-13) * x - 2.2212e-8) *
        x + 6.3885e-4) * x + 3.7199e+1) +
        exp(-2.1512e-4 * x + 41.491);
}

fn atmosphereDensity(height: f32) -> vec3f {
    return vec3f(
        airDensity(height - earthRadius),
        aerosolDensity(height - earthRadius),
        ozoneDensity(height - earthRadius),
    );
}

fn maxAtmosphereDensity() -> vec3f {
    return vec3f(
        airDensity(0.0),
        aerosolDensity(0.0),
        ozoneDensity(0.0),
    );
}

fn airRefractiveIndex(lambda: f32) -> f32 {
    let x = lambda / 1000.0;
    let ns = 5791817.0 / (238.0185 - 1.0 / (x * x)) + 167909.0 / (57.362 - 1.0 / (x * x));
    return ns / 1.0e8 + 1.0;
}

fn depolarizationFactor(lambda: f32) -> f32 {
    let x = lambda / 1000.0;
    return ((((-5.20398 * x + 8.31652) * x - 2.3355) * x - 2.0086) * x + 3.72696) * 1.0e-2;
}

fn rayleighScatteringBeta(lambda: f32, n: f32) -> f32 {
    const N: f32 = 2.545e25;

    let pn = depolarizationFactor(lambda);

    let a = n * n - 1.0;
    let k = (6.0 + 3.0 * pn) / (6.0 - 7.0 * pn);

    let lambda2 = (lambda * 1.0e-9) * (lambda * 1.0e-9);
    return k * 8.0 * PI * PI * PI * a * a / (3.0 * N * lambda2 * lambda2);
}

fn mieScatteringBeta(lambda: f32, turbidity: f32) -> f32 {
    const v: f32 = 4.0;

    let c = (0.6544 * turbidity - 0.6510) * 1.0e-16;

    let x = lambda / 1000.0;
    let K = (((-1.26852 * x + 3.51896) * x - 3.76305) * x + 1.87791) * x + 0.313775;

    return 0.434 * c * PI * pow(2.0 * PI / (lambda * 1.0e-9), v - 2.0) * K;
}

fn rayleighPhase(cosTheta: f32, lambda: f32) -> f32 {
    let pn = depolarizationFactor(lambda);
    let gamma = pn / (2.0 - pn);
    return 3.0 / (16.0 * PI * (1.0 + 2.0 * gamma)) * (1.0 + 3.0 * gamma + (1.0 - gamma) * cosTheta * cosTheta);
}

fn hgDraineParams(d: f32, gHG: ptr<function, f32>, gD: ptr<function, f32>, a: ptr<function, f32>, wD: ptr<function, f32>) {
    if (d <= 0.1) {
        *gHG = 13.8 * d * d;
        *gD = 1.1456 * d * sin(9.29044 * d);
        *a = 250.0;
        *wD = 0.252977 - 312.983 * pow(d, 4.3);
    } else if (d < 1.5) {
        let logd = log(d);
        *gHG = 0.862 - 0.143 * logd * logd;
        *gD = 0.379685 * cos(1.19692 *
            cos((logd - 0.238604) * (logd + 1.00667) / (0.507522 - 0.15677 * logd)) +
            1.37932 * logd + 0.0625835) + 0.344213;
        *a = 250.0;
        *wD = 0.146209 * cos(3.38707 * logd + 2.11193) + 0.316072 + 0.0778917 * logd;
    } else if (d < 5.0) {
        let logd = log(d);
        *gHG = 0.0604931 * log(logd) + 0.940256;
        *gD = 0.500411 - 0.081287 / (-2.0 * logd + tan(logd) + 1.27551);
        *a = 7.30354 * logd + 6.31675;
        *wD = 0.026914 * (logd - cos(5.68947 * (log(logd) - 0.0292149))) + 0.376475;
    } else if (d < 50.0) {
        *gHG = exp(-0.0990567 / (d - 1.67154));
        *gD = exp(-2.20679 / (d + 3.91029) - 0.428934);
        *a = exp(3.62489 - 8.29288 / (d + 5.52825));
        *wD = exp(-0.599085 / (d - 0.641583) - 0.665888);
    }
}

fn drainePhase(cosTheta: f32, a: f32, g: f32) -> f32 {
    let n1 = (1.0 - g * g) / pow(1.0 + g * g - 2.0 * g * cosTheta, 1.5);
    let n2 = (1.0 + a * cosTheta * cosTheta) / (1.0 + a * (1.0 + 2.0 * g * g) / 3.0);
    return n1 * n2 / (4.0 * PI);
}

fn hgDrainePhase(cosTheta: f32, d: f32) -> f32 {
    var gHG: f32;
    var gD: f32;
    var a: f32;
    var wD: f32;
    hgDraineParams(d, &gHG, &gD, &a, &wD);

    return (1.0 - wD) * drainePhase(cosTheta, 0.0, gHG) + wD * drainePhase(cosTheta, a, gD);
}

fn sampleHenyeyGreenstein(rand: f32, g: f32) -> f32 {
    let t = (1.0 - g * g) / (1.0 - g + 2.0 * g * rand);
    return (1.0 + g * g - t) / (2.0 * g);
}

fn sampleDraine(rand: f32, g: f32, a: f32) -> f32 {
    let t0 = a - a * g * g;
    let t1 = a * g * g * g * g - a;
    let t2 = -3.0 * (4.0 * (g * g * g * g - g * g) + t1 * (1.0 + g * g));
    let t3 = g * (2.0 * rand - 1.0);
    let t4 = 3.0 * g * g * (1.0 + t3) + a * (2.0 + g * g * (1.0 + (1.0 + 2.0 * g * g) * t3));
    let t5 = t0 * (t1 * t2 + t4 * t4) + t1 * t1 * t1;
    let t6 = t0 * 4.0 * (g * g * g * g - g * g);
    let t7 = pow((t5 + sqrt(t5 * t5 - t6 * t6 * t6)), 1.0 / 3.0);
    let t8 = 2.0 * (t1 + t6 / t7 + t7) / t0;
    let t9 = sqrt(6.0 * (1.0 + g * g) + t8);
    return g / 2.0 + (1.0 / (2.0 * g) - 1.0 / (8.0 * g) * pow(
        sqrt(6.0 * (1.0 + g * g) - t8 + 8.0 * t4 / (t0 * t9)) - t9, 2.0));
}

fn sampleFromDeflectionCosine(w: vec3f, cosTheta: f32, rand: f32) -> vec3f {
    let sinTheta = sqrt(1.0 - cosTheta * cosTheta);
    let phi = 2.0 * PI * rand;
    let spherical = vec3(sinTheta * cos(phi), sinTheta * sin(phi), cosTheta);

    var b1: vec3f;
    var b2: vec3f;
    buildOrthonormalBasis(w, &b1, &b2);

    return b1 * spherical.x + b2 * spherical.y + w * spherical.z;
}

fn sampleHgDraine(w: vec3f, rand: vec3f, d: f32) -> vec3f {
    var gHG: f32;
    var gD: f32;
    var a: f32;
    var wD: f32;
    hgDraineParams(d, &gHG, &gD, &a, &wD);

    var cosTheta: f32;
    if (rand.y < wD) {
        cosTheta = sampleDraine(rand.x, gD, a);
    } else {
        cosTheta = sampleHenyeyGreenstein(rand.x, gHG);
    }

    return sampleFromDeflectionCosine(w, cosTheta, rand.z);
}

fn kleinNishinaPhase(cosTheta: f32, e: f32) -> f32 {
    return e / (2.0 * PI * (e * (1.0 - cosTheta) + 1.0) * log(2.0 * e + 1.0));
}

fn sampleKleinNishina(w: vec3f, rand: vec2f, e: f32) -> vec3f {
    let cosTheta = (-pow(2.0 * e + 1.0, 1.0 - rand.x) + e + 1.0) / e;
    return sampleFromDeflectionCosine(w, cosTheta, rand.y);
}

fn ozoneAbsorption(x: f32) -> f32{
    if (x > 213.0 && x < 380.0) {
        return exp(-1632.43483 + x * (27.2816384 +
            x * (-0.188136709 + x * (0.000652848908 +
            x * (-1.13680875e-06 + x * (7.90050380e-10))))));
    }
    if (x >= 380.0 && x <= 780.0) {
        return exp(-178.194363 + x * (1.07246495 +
            x * (-0.00403429758 + x * (8.14291496e-06 +
            x * (-8.30370951e-09 + x * 3.31168412e-12))))) +
            exp(0.1 * min(603.0 - x, 0.0) + 0.02 * x - 60.414287) *
            (exp(0.5 * cos(0.19039955476 * x - 114.810931522)) - 1.0);
    }
    if (x > 780.0) {
        return exp(-36.6867555 - x * 0.0162778335);
    }
    return 0.0;
}

fn atmosphereExtinctionBeta(wavelength: f32) -> vec3f {
    let ns = airRefractiveIndex(wavelength);

    let betaR = rayleighScatteringBeta(wavelength, ns);
    let betaM = mieScatteringBeta(wavelength, atmosphereTurbidity);
    let sigmaO = ozoneAbsorption(wavelength);

    return vec3f(betaR, 1.1 * betaM, 0.0001 * sigmaO);
}

fn convertToEarthSpace(x: vec3f) -> vec3f {
    return vec3f(x.x, x.y + earthRadius, x.z);
}

fn convertRayToEarthSpace(r: ray) -> ray {
    return ray(convertToEarthSpace(r.origin), r.direction);
}

fn intersectSphere(r: ray, center: vec3f, radius: f32) -> vec2f {
    let oc = r.origin - center;
    let b = dot(oc, r.direction);
    let qc = oc - b * r.direction;
    var h = radius * radius - dot(qc, qc);
    if (h < 0.0) {
        return vec2(-1.0);
    }

    h = sqrt(h);
    return vec2(-b - h, -b + h);
}

fn estimateTransmittance(r_: ray, beta: vec3f) -> f32 {
    var r = r_;

    let earthDist = intersectSphere(r, vec3(0.0), earthRadius);
    let atmosphereDist = intersectSphere(r, vec3(0.0), atmosphereRadius);

    if (atmosphereDist.y < 0.0) {
        return 1.0;
    }
    if (earthDist.y >= 0.0) {
        return 0.0;
    }

    var t = max(0.0, atmosphereDist.x);
    r.origin += r.direction * t;

    let betaMax = beta * maxAtmosphereDensity();
    let betaSum = betaMax.x + betaMax.y + betaMax.z;

    var transmittance = 1.0;
    while (t < atmosphereDist.y) {
        let flightDistance = -log(1.0 - random1()) / betaSum;

        r.origin += r.direction * flightDistance;
        t += flightDistance;

        let height = length(r.origin);

        let betaH = beta * atmosphereDensity(height) / betaSum;
        transmittance *= 1.0 - betaH.x - betaH.y - betaH.z;
    }

    return transmittance;
}

fn compositeDeltaTracking(r_: ray, beta: vec3f) -> vec2f {
    var r = r_;

    let atmosphereDist = intersectSphere(r, vec3(0.0), atmosphereRadius);
    if (atmosphereDist.y < 0.0) {
        return vec2(-1.0);
    }

    var t = max(0.0, atmosphereDist.x);
    r.origin += r.direction * t;

    let betaMax = beta * maxAtmosphereDensity();
    let betaSum = betaMax.x + betaMax.y + betaMax.z;

    for (var i: i32 = 0; i < 1024; i++) {
        let flightDistance = -log(1.0 - random1()) / betaSum;

        r.origin += r.direction * flightDistance;
        t += flightDistance;

        let height = length(r.origin);
        if (clamp(height, earthRadius, atmosphereRadius) != height) {
            return vec2(-1.0);
        }

        let betaH = beta * atmosphereDensity(height) / betaSum;

        let r = random1();
        if (r >= betaH.x + betaH.y + betaH.z) {
            continue; // Null collision
        }

        var particle = 2.0;
        if (r < betaH.x) {
            particle = 0.0;
        } else if (r < betaH.x + betaH.y) {
            particle = 1.0;
        }

        return vec2(t, particle);
    }

    return vec2(-1.0);
}

fn pathTraceAtmosphere(r_: ray, sunPosition: vec3f, sunRadiance: f32, beta: vec3f, wavelength: f32) -> f32 {
    var r = r_;

    var L = 0.0;
    var throughput = 1.0;

    for (var i: i32 = 0; i < 64; i++) {
        let interaction = compositeDeltaTracking(r, beta);
        if (interaction.x < 0.0) {
            break;
        }

        r.origin += r.direction * interaction.x;

        if ((interaction.y == 2.0) || // Ozone
            (interaction.y == 1.0 && random1() > (1.0 / 1.1))) { // Aerosols
            break; // Absorption
        }

        var weight: f32;
        let sunDirection = sampleSunDirection(random2(), sunPosition, r.origin, &weight);

        let transmittance = estimateTransmittance(ray(r.origin, sunDirection), beta);

        var wo: vec3f;
        var phaseLight: f32;
        var estimator: f32;
        if (interaction.y == 0.0) { // Air molecules
            phaseLight = rayleighPhase(dot(r.direction, sunDirection), wavelength);

            wo = sampleSphere(random2());
            estimator = 4.0 * PI * rayleighPhase(dot(r.direction, wo), wavelength);
        } else if (interaction.y == 1.0) { // Aerosols
            const energyParameter = 3000.0;
            phaseLight = kleinNishinaPhase(dot(r.direction, sunDirection), energyParameter);

            wo = sampleKleinNishina(r.direction, random2(), energyParameter);
            estimator = 1.0;
        }

        r.direction = wo;

        L += throughput * weight * transmittance * phaseLight * sunRadiance;
        throughput *= estimator;

        let exitProbability = clamp(throughput, 0.0, 1.0);
        if (random1() > exitProbability) {
            break;
        }
        throughput /= exitProbability;
    }

    return L;
}

fn spectrumToXYZ(lambda: f32, power: f32) -> vec3f {
    return (CIE_CMF_XYZ(lambda) * max(power, 0.0) * 683.0);
}

fn lrgbToReflectanceSpectrum(lambda: f32, rgb: vec3f) -> f32 {
    return dot(rgb, CIE_BT709_Basis(lambda));
}

@compute
@workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) dispatch_id: vec3u) {
    let texture_dim = textureDimensions(output_texture);
    if (dispatch_id.x >= texture_dim.x || dispatch_id.y >= texture_dim.y) {
        return;
    }

    let pixel_center = vec2f(dispatch_id.xy);
    var uv = pixel_center / vec2f(texture_dim);

    rng_state.seed = vec3f(uv, f32(camera.frame_index));
    uv += random2() / vec2f(texture_dim);

    let ndc = vec2f(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);

    let origin = (camera.view_inverse * vec4f(0.0, 0.0, 0.0, 1.0)).xyz;
    let view_direction = camera.projection_inverse * vec4f(ndc, 1.0, 1.0);
    let direction = normalize((camera.view_inverse * vec4f(normalize(view_direction.xyz), 0.0)).xyz);

    var r = ray(origin, direction);

    var L = 0.0;
    var throughput = 1.0;

    const depth_limit = 16;

    var lambda_pdf = 0.0;
    let lambda = sampleWavelength(random1(), &lambda_pdf);

    for (var i = 0; i <= depth_limit; i++) {
        if (i == depth_limit) {
            throughput = 0.0;
            break;
        }

        var query: ray_query;
        rayQueryInitialize(&query, scene, RayDesc(0u, 0xFFu, 1.0e-3, 1.0e6, r.origin, r.direction));
        rayQueryProceed(&query);

        let intersection = rayQueryGetCommittedIntersection(&query);
        if (intersection.kind != RAY_QUERY_INTERSECTION_NONE) {
            let instance_data = instance_buffer[intersection.instance_index];

            let index0 = index_buffer[instance_data.first_index + intersection.primitive_index * 3];
            let index1 = index_buffer[instance_data.first_index + intersection.primitive_index * 3 + 1];
            let index2 = index_buffer[instance_data.first_index + intersection.primitive_index * 3 + 2];

            let vertex0 = vertex_buffer[instance_data.first_vertex + index0];
            let vertex1 = vertex_buffer[instance_data.first_vertex + index1];
            let vertex2 = vertex_buffer[instance_data.first_vertex + index2];
            let barycentrics = barycentricWeights(intersection.barycentrics);

            let uv0 = vec2f(vertex0.position_u.w, vertex0.normal_v.w);
            let uv1 = vec2f(vertex1.position_u.w, vertex1.normal_v.w);
            let uv2 = vec2f(vertex2.position_u.w, vertex2.normal_v.w);
            let has_uvs = all(uv0 >= vec2f(MISSING_UV_COORD + 1.0)) &&
                all(uv1 >= vec2f(MISSING_UV_COORD + 1.0)) &&
                all(uv2 >= vec2f(MISSING_UV_COORD + 1.0));

            let mesh_uv = barycentrics.x * uv0 + barycentrics.y * uv1 + barycentrics.z * uv2;
            let texture_uv = select(intersection.barycentrics, mesh_uv, has_uvs);

            var geometric_normal = normalize(intersection.object_to_world * vec4f(cross(
                vertex0.position_u.xyz - vertex1.position_u.xyz,
                vertex0.position_u.xyz - vertex2.position_u.xyz), 0.0));
            geometric_normal *= -sign(dot(geometric_normal, r.direction));
            let mesh_normal = barycentrics.x * vertex0.normal_v.xyz +
                barycentrics.y * vertex1.normal_v.xyz +
                barycentrics.z * vertex2.normal_v.xyz;
            let has_mesh_normal = dot(mesh_normal, mesh_normal) > 0.0;

            var normal = geometric_normal;
            if (has_mesh_normal) {
                normal = normalize(intersection.object_to_world * vec4f(mesh_normal, 0.0));
            }

            var w0 = vec3f(0.0);
            var w1 = vec3f(0.0);
            buildOrthonormalBasis(normal, &w0, &w1);

            var material = instance_data.material.material1;
            let mix_factor = resolveTextureOr(
                instance_data.material.mix_factor,
                TextureSamplingContext(texture_uv),
                vec3f(0.0),
            ).r;
            if (random1() < mix_factor) {
                material = instance_data.material.material2;
            }

            let normal_mapping = resolveTextureOr(
                material.normal_map,
                TextureSamplingContext(texture_uv),
                vec3f(0.5, 0.5, 1.0),
            );

            normal = normalize((normal_mapping.x * 2.0 - 1.0) * w0 + (normal_mapping.y * 2.0 - 1.0) * w1 + normal_mapping.z * normal);
            normal *= -sign(dot(normal, r.direction));

            r.origin += r.direction * intersection.t;
            r.origin += geometric_normal * 1.0e-3;

            let albedo = resolveTextureOr(
                material.reflectance,
                TextureSamplingContext(texture_uv),
                vec3f(0.8),
            );
            let roughness = resolveTextureOr(
                material.roughness_u,
                TextureSamplingContext(texture_uv),
                vec3f(1.0),
            ).r;
            let eta = resolveTextureOr(
                material.eta,
                TextureSamplingContext(texture_uv),
                vec3f(1.33),
            ).r;

            var n0 = 1.0;
            var n1 = eta;

            var microfacet_normal = normal;
            if (roughness > 0.01) {
                microfacet_normal = sampleVndf(random2(), -r.direction, roughness, normal);
            }
            var reflectance = fresnelDielectric(dot(-r.direction, microfacet_normal), n0, n1);
            if (material.material_flags == 2) {
                reflectance = 1.0;
            }

            if (random1() <= reflectance) {
                r.direction = reflect(r.direction, microfacet_normal);
                if (material.material_flags == 2) {
//                    throughput *= fresnelConductor(dot(-r.direction, microfacet_normal), vec3f(1.34560, 0.96521, 0.61722), vec3f(7.47460, 6.39950, 5.30310));
                }
            } else {
                throughput *= lrgbToReflectanceSpectrum(lambda, albedo);

                r.direction = sampleCosineHemisphereAround(random2(), normal);
            }

            if (dot(r.direction, geometric_normal) < 0.0) {
                r.direction = reflect(r.direction, geometric_normal);
            }

            r.direction = normalize(r.direction);
        } else {
            break;
        }
    }

    let sun_direction = normalize(vec3f(1.0, 1.0, 1.0));
    let sun_position = sun_direction * astronomicalUnit;
    let sun_radiance = getSunRadiance(lambda);
    let extinction_beta = atmosphereExtinctionBeta(lambda);
    L += throughput * pathTraceAtmosphere(convertRayToEarthSpace(r), sun_position, sun_radiance, extinction_beta, lambda);

    L /= lambda_pdf;

    var L_xyz = spectrumToXYZ(lambda, L);
    let history = textureLoad(accumulation_buffer, dispatch_id.xy).xyz;
    L_xyz = mix(history, L_xyz, 1.0 / max(f32(camera.frame_index), 1.0));

    textureStore(accumulation_buffer, dispatch_id.xy, vec4f(L_xyz, 1.0));

    let color = XYZ_TO_sRGB * L_xyz / 100000.0;
    textureStore(output_texture, dispatch_id.xy, vec4f(acesFitted(color), 1.0));
}
