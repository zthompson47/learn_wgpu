struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct RotationUniform {
    view_proj: mat4x4<f32>,
};
@group(2) @binding(0)
var<uniform> rotation: RotationUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = rotation.view_proj * camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}

/*
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //return vec4<f32>(0.3, 0.2, 0.1, 1.0);
    return vec4<f32>(in.color, 1.0);
}
*/

/*
@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y,  0.0, 1.0);
    // Brown is the color.
    out.color = vec4<f32>(0.3, 0.2, 0.1, 1.0);
    return out;
}

@vertex
fn vs_main2(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y,  0.0, 1.0);
    // Choose color based on vertex positions.
    out.color = vec4<f32>(x, y, 1.0, 1.0);
    return out;
}
*/
/*
@vertex
fn vs_main2(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    //out.color = model.color; //TODO - make this different
    out.color = vec3<f32>(model.color[2], model.color[1], model.color[0]);
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}
*/
