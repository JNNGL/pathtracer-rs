enable wgpu_ray_query;

struct CameraUniform {
    view_inverse: mat4x4<f32>,
    projection_inverse: mat4x4<f32>,
}

@group(0) @binding(0)
var output_texture: texture_storage_2d<rgba16float, write>;

@group(0) @binding(2)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var scene: acceleration_structure;

@compute
@workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) dispatch_id: vec3u) {
    let texture_dim = textureDimensions(output_texture);
    if (dispatch_id.x >= texture_dim.x || dispatch_id.y >= texture_dim.y) {
        return;
    }

    let pixel_center = vec2f(dispatch_id.xy) + vec2f(0.5);
    let uv = pixel_center / vec2f(texture_dim);
    let ndc = vec2f(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);

    let origin = (camera.view_inverse * vec4f(0.0, 0.0, 0.0, 1.0)).xyz;
    let view_direction = camera.projection_inverse * vec4f(ndc, 1.0, 1.0);
    let direction = normalize((camera.view_inverse * vec4f(normalize(view_direction.xyz), 0.0)).xyz);

    var query: ray_query;
    rayQueryInitialize(&query, scene, RayDesc(0u, 0xFFu, 1.0e-3, 1.0e6, origin, direction));
    rayQueryProceed(&query);

    var color = vec3f(0.0, 0.0, 0.0);

    let intersection = rayQueryGetCommittedIntersection(&query);
    if (intersection.kind != RAY_QUERY_INTERSECTION_NONE) {
        color = vec3f(intersection.barycentrics, 1.0 - intersection.barycentrics.x - intersection.barycentrics.y);
    }

    textureStore(output_texture, dispatch_id.xy, vec4f(color, 1.0));
}
