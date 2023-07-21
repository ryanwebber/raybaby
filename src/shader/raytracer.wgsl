
struct Globals {
    camera: Camera, 
}

struct Camera {
    focal_plane: vec3<f32>,
    world_space_position: vec3<f32>,
    local_to_world_matrix: mat4x4<f32>,
    near_clip: f32,
    far_clip: f32,
}

struct Material {
    color: vec4<f32>,
}

struct MaterialBuffer {
    count: u32,
    materials: array<Material>,
}

struct Sphere {
    position: vec3<f32>,
    radius: f32,
    material_id: u32,

    // 16 byte alignment
    _padding: array<u32, 3>,
}

struct SphereBuffer {
    count: u32,
    spheres: array<Sphere>,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
};

struct HitInfo {
    hit: bool,
    distance: f32,
    position: vec3<f32>,
    normal: vec3<f32>,
}

fn ray_sphere_intersection(ray: Ray, center: vec3<f32>, radius: f32) -> HitInfo {
    var hit: HitInfo;
    hit.hit = false;

    // compute if the ray intersects the sphere
    let oc = ray.origin - center;
    let a = dot(ray.direction, ray.direction);
    let b = 2.0 * dot(oc, ray.direction);
    let c = dot(oc, oc) - radius * radius;

    let discriminant = b * b - 4.0 * a * c;

    if (discriminant >= 0.0) {
        var temp = (-b - sqrt(discriminant)) / (2.0 * a);
        if (temp < 0.0) {
            temp = (-b + sqrt(discriminant)) / (2.0 * a);
        }

        if (temp > 0.0) {
            hit.hit = true;
            hit.distance = temp;
            hit.position = ray.origin + ray.direction * temp;
            hit.normal = (hit.position - center) / radius;
        }
    }

    return hit;
}

fn ray_world_collision(ray: Ray) -> HitInfo {
    var hit: HitInfo;
    hit.hit = false;
    hit.distance = globals.camera.far_clip;

    let spheres = &sphere_buffer.spheres;
    for (var i: u32 = 0u; i < sphere_buffer.count; i++) {
        let sphere_pos = (*spheres)[i].position;
        let sphere_radius = (*spheres)[i].radius;
        let hit_info = ray_sphere_intersection(ray, sphere_pos, sphere_radius);
        if (hit_info.hit && hit_info.distance < hit.distance) {
            hit = hit_info;
        }
    }

    return hit;
}

fn trace(ray: Ray) -> vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

@group(0) @binding(0)
var<uniform> globals: Globals;

@group(0) @binding(1)
var tex: texture_storage_2d<rgba32float, write>;

@group(0) @binding(2)
var<storage, read> mat_buffer: MaterialBuffer;

@group(0) @binding(3)
var<storage, read> sphere_buffer: SphereBuffer;

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) g_invocation_id: vec3<u32>) {
    let dimensions = textureDimensions(tex);
    let uv = vec2<f32>(
        f32(g_invocation_id.x) / f32(dimensions.x),
        f32(g_invocation_id.y) / f32(dimensions.y)
    );

    let focus_point_local = vec3<f32>(uv - 0.5, 1.0) * globals.camera.focal_plane;
    let focusPoint = globals.camera.local_to_world_matrix * vec4<f32>(focus_point_local, 1.0);
    let cam_right = globals.camera.local_to_world_matrix[0].xyz;
    let cam_up = globals.camera.local_to_world_matrix[1].xyz;

    var ray: Ray;
    ray.origin = globals.camera.world_space_position;
    ray.direction = normalize(focusPoint.xyz - ray.origin);
    
    var color = vec3<f32>(0.0, 0.0, 0.0);

    let hit = ray_world_collision(ray);
    if (hit.hit) {
        let dp = dot(-ray.direction, hit.normal);
        color = ray.direction * dp;
    }

    textureStore(tex, g_invocation_id.xy, vec4<f32>(color, 1.0));
}
