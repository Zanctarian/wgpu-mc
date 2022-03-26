struct Uniforms {
    view_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniform_data: Uniforms;

struct VertexResult {
    [[builtin(position)]] pos: vec4<f32>;
    [[location(0)]] color: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] pos_in: vec3<f32>,
    [[location(1)]] color: u32
) -> VertexResult {
    var vr: VertexResult;

    vr.pos = uniform_data.view_proj * vec4<f32>(pos_in, 1.0);

    vr.color = vec3<f32>(f32((color << 16u) & 0xffu) / 255.0, f32((color << 8u) & 0xffu) / 255.0, f32(color & 0xffu) / 255.0);

    return vr;
}

[[stage(fragment)]]
fn fs_main(in: VertexResult) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}