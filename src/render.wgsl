enable wgpu_ray_query;

const PI: f32 = 3.14159265358979323846;

struct CameraUniform {
    view_inverse: mat4x4<f32>,
    projection_inverse: mat4x4<f32>,
    frame_index: u32,
}

struct VertexBuffer {
    position_u: vec4f,
    normal_v: vec4f,
}

struct InstanceData {
    first_index: u32,
    first_vertex: u32,
}

@group(0) @binding(0)
var output_texture: texture_storage_2d<rgba16float, write>;

@group(0) @binding(1)
var accumulation_buffer: texture_storage_2d<rgba32float, read_write>;

@group(0) @binding(2)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var scene: acceleration_structure;

@group(1) @binding(1)
var<storage, read> vertex_buffer: array<VertexBuffer>;

@group(1) @binding(2)
var<storage, read> index_buffer: array<u32>;

@group(1) @binding(3)
var<storage, read> instance_buffer: array<InstanceData>;

struct RNGState {
    seed: vec3f,
}

var<private> rng_state: RNGState;

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


    var origin = (camera.view_inverse * vec4f(0.0, 0.0, 0.0, 1.0)).xyz;
    let view_direction = camera.projection_inverse * vec4f(ndc, 1.0, 1.0);
    var direction = normalize((camera.view_inverse * vec4f(normalize(view_direction.xyz), 0.0)).xyz);

    var throughput = vec3f(1.0);
    var color = vec3f(0.0);

    for (var i = 0; i < 5; i++) {
        var query: ray_query;
        rayQueryInitialize(&query, scene, RayDesc(0u, 0xFFu, 1.0e-3, 1.0e6, origin, direction));
        rayQueryProceed(&query);

        let intersection = rayQueryGetCommittedIntersection(&query);
        if (intersection.kind != RAY_QUERY_INTERSECTION_NONE) {
            let instance_data = instance_buffer[intersection.instance_index];

            let index0 = index_buffer[instance_data.first_index + intersection.primitive_index * 3];
            let index1 = index_buffer[instance_data.first_index + intersection.primitive_index * 3 + 1];
            let index2 = index_buffer[instance_data.first_index + intersection.primitive_index * 3 + 2];

            let vertex0 = vertex_buffer[instance_data.first_vertex + index0].position_u.xyz;
            let vertex1 = vertex_buffer[instance_data.first_vertex + index1].position_u.xyz;
            let vertex2 = vertex_buffer[instance_data.first_vertex + index2].position_u.xyz;

            var normal = normalize(intersection.object_to_world * vec4f(cross(vertex0 - vertex1, vertex0 - vertex2), 0.0));
            normal *= -sign(dot(normal, direction));

//            color = normal;

            origin += direction * intersection.t;
            origin += normal * 1.0e-3;

            direction = sampleCosineHemisphereAround(random2(), normal);

            throughput *= 0.8;
        } else {
            color = throughput * 4.0;
            break;
        }
    }

    let history = textureLoad(accumulation_buffer, dispatch_id.xy).xyz;
    color = mix(history, color, 1.0 / max(f32(camera.frame_index), 1.0));

    textureStore(accumulation_buffer, dispatch_id.xy, vec4f(color, 1.0));

    textureStore(output_texture, dispatch_id.xy, vec4f(color, 1.0));
}
