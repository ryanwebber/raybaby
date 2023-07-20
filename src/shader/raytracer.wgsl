
@group(0) @binding(1)
var tex: texture_storage_2d<rgba32float, write>;

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) g_invocation_id: vec3<u32>) {
    let dimensions = textureDimensions(tex);
    let color = vec4<f32>(
        f32(g_invocation_id.x) / f32(dimensions.x),
        f32(g_invocation_id.y) / f32(dimensions.y),
        0.0,
        1.0
    );

    textureStore(tex, g_invocation_id.xy, color);
}
