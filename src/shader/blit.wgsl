struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.vert_pos = out.clip_position.xyz;
    return out;
}

@group(0) @binding(0)
var tex: texture_storage_2d<rgba32float, read>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dimension = textureDimensions(tex);
    let tex_coords = vec2<u32>(
        u32((in.vert_pos.x + 0.5) * f32(dimension.x)),
        u32((in.vert_pos.y + 0.5) * f32(dimension.y)),
    );

    let color = textureLoad(tex, tex_coords);
    return vec4<f32>(color.xyz, 1.0);
}
