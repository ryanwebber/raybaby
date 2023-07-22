
// ============================= Structures ============================ */

alias RandomState = ptr<function, u32>;

struct Globals {
    camera: Camera,
    frame: u32,
    random_seed: u32,
    skybox_color: vec3<f32>,
    ambient_lighting_color: vec3<f32>,
    ambient_lighting_strength: f32,
    max_ray_bounces: u32,
    max_samples_per_pixel: u32,
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
    luminosity: f32,
    _padding: array<u32, 3>,
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
    material_id: u32,
}

// ============================= Ray Tracing Logic ============================ */

fn get_environment_lighting() -> vec3<f32> {
    return globals.ambient_lighting_color * globals.ambient_lighting_strength;
}

fn ray_sphere_intersection(ray: Ray, sphere: Sphere) -> HitInfo {

    let center = sphere.position;
    let radius = sphere.radius;

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

        if (temp >= 0.0) {
            hit.hit = true;
            hit.distance = temp;
            hit.position = ray.origin + ray.direction * temp;
            hit.normal = normalize(hit.position - center);
            hit.material_id = sphere.material_id;
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
        let hit_info = ray_sphere_intersection(ray, (*spheres)[i]);
        if (hit_info.hit && hit_info.distance < hit.distance) {
            hit = hit_info;
        }
    }

    return hit;
}

fn trace(ray: Ray, rs: RandomState) -> vec3<f32> {
    var light = vec3<f32>(0.0);
    var ray_color = vec3<f32>(1.0);

    var ray = ray;
    var bounces: u32 = 0u;
    while (bounces <= globals.max_ray_bounces) {
        let hit_info = ray_world_collision(ray);
        if (!hit_info.hit) {
            if (bounces > 0u) {
                let env_light = get_environment_lighting();
                light += vec3<f32>(
                    env_light.x * ray_color.x,
                    env_light.y * ray_color.y,
                    env_light.z * ray_color.z
                );
            } else {
                light = globals.skybox_color;
            }

            break;
        }

        let diffuse_random = random_unit_vector(rs);
        let diffuse_reflection = select(
            -diffuse_random,
            diffuse_random,
            dot(diffuse_random, hit_info.normal) > 0.0
        );

        let specular_reflection = reflect(ray.direction, hit_info.normal);

        ray.origin = hit_info.position;
        ray.direction = normalize(mix(diffuse_reflection, specular_reflection, 0.0));

        bounces++;

        let mat = mat_buffer.materials[hit_info.material_id];

        let emission = mat.color.xyz * mat.luminosity;
        let emission_strength = dot(hit_info.normal, ray.direction);
        light += vec3<f32>(
            emission.x * ray_color.x,
            emission.y * ray_color.y,
            emission.z * ray_color.z
        );

        ray_color = vec3<f32>(
            ray_color.x * mat.color.x,
            ray_color.y * mat.color.y,
            ray_color.z * mat.color.z
        ) * emission_strength * 2.0;

        let p = max(ray_color.x, max(ray_color.y, ray_color.z));
        if (random_value(rs) >= p) {
            break;
        }

        ray_color /= p;
    }

    return light;
}

// ============================= Randomness ============================ */

fn random_next(rs: RandomState) -> u32 {
    // PCG (permuted congruential generator). Thanks to:
    // www.pcg-random.org and www.shadertoy.com/view/XlGcRh

    *rs = *rs * 747796405u + 2891336453u;
    var result: u32 = ((*rs >> ((*rs >> 28u) + 4u)) ^ *rs) * 277803737u;
    result = (result >> 22u) ^ result;

    return result;
}

fn random_value(rs: RandomState) -> f32 {
    return f32(random_next(rs)) / 4294967295.0;
}

fn random_normally_distributed_value(rs: RandomState) -> f32
{
    // https://stackoverflow.com/a/6178290
    let theta = 2.0 * 3.1415926 * random_value(rs);
    let rho = sqrt(-2.0 * log(random_value(rs)));
    return rho * cos(theta);
}

fn random_unit_vector(rs: RandomState) -> vec3<f32>
{
    // https://math.stackexchange.com/a/1585996
    let x = random_normally_distributed_value(rs);
    let y = random_normally_distributed_value(rs);
    let z = random_normally_distributed_value(rs);
    return normalize(vec3<f32>(x, y, z));
}

// ============================= Entry Point ============================ */

@group(0) @binding(0)
var<uniform> globals: Globals;

@group(0) @binding(1)
var tex: texture_storage_2d<rgba32float, read_write>;

@group(0) @binding(2)
var<storage, read> mat_buffer: MaterialBuffer;

@group(0) @binding(3)
var<storage, read> sphere_buffer: SphereBuffer;

@compute
@workgroup_size(1, 1, 1)
fn main(
    @builtin(global_invocation_id) g_invocation_id: vec3<u32>
) {

    let dimensions = textureDimensions(tex);
    let pixel_index = g_invocation_id.y * dimensions.x + g_invocation_id.x;
    let pixel_coords = vec2<u32>(g_invocation_id.x, g_invocation_id.y);

    var rs: u32 = pixel_index + globals.frame * 719393u + globals.random_seed;

    let uv = vec2<f32>(
        f32(pixel_coords.x) / f32(dimensions.x),
        f32(pixel_coords.y) / f32(dimensions.y)
    );

    let focus_point_local = vec3<f32>(uv - 0.5, 1.0) * globals.camera.focal_plane;
    let focusPoint = globals.camera.local_to_world_matrix * vec4<f32>(focus_point_local, 1.0);
    let cam_right = globals.camera.local_to_world_matrix[0].xyz;
    let cam_up = globals.camera.local_to_world_matrix[1].xyz;

    let num_samples = min(globals.max_samples_per_pixel, 100u);

    var ray: Ray;
    var color = vec3<f32>(0.0);
    for (var i: u32 = 0u; i < num_samples; i++) {
        ray.origin = globals.camera.world_space_position;
        ray.direction = normalize(focusPoint.xyz - ray.origin);
        color += trace(ray, &rs);
    }
    
    color /= max(f32(num_samples), 1.0);

    let weight = 1.0 / (f32(globals.frame) / 4.0 + 1.0);
    let previous_color = textureLoad(tex, g_invocation_id.xy).xyz;
    let color_average = saturate(previous_color * (1.0 - weight) + color * weight);

    textureStore(tex, g_invocation_id.xy, vec4<f32>(color_average, 1.0));
}
