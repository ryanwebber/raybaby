struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    return out;
}

@group(0) @binding(0)
var tex: texture_storage_2d<rgba32float, read>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dimension = textureDimensions(tex);
    let tex_coords = vec2<u32>(
        u32(in.uv.x * f32(dimension.x)),
        u32(in.uv.y * f32(dimension.y)),
    );

    let color = textureLoad(tex, tex_coords);
    return color;
}
