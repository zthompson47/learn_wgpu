struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct RotationUniform {
    view_proj: mat4x4<f32>,
};
@group(2) @binding(0)
var<uniform> rotation: RotationUniform;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}
@group(3) @binding(0)
var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );

    // FIXME Not sure if it's build by column vectors..
    // Seems to work though..
    let rotation_3x3 = mat3x3<f32>(
        rotation.view_proj[0].xyz,
        rotation.view_proj[1].xyz,
        rotation.view_proj[2].xyz,
    );
    // Below looks off, so probably column-based..
    //let rotation_3x3_2 = transpose(rotation_3x3);

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    out.world_normal =
        (normal_matrix * rotation_3x3)
        * model.normal;
    // OR, if uniform scaling only:
    //out.world_normal = (
    //    (model_matrix * rotation.view_proj)
    //    * vec4<f32>(model.normal, 0.0)
    //).xyz;

    var world_position: vec4<f32> =
        model_matrix
        * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;

    out.clip_position =
        camera.view_proj
        * model_matrix
        * rotation.view_proj
        * vec4<f32>(model.position, 1.0);

    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var t_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //return textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let object_normal: vec4<f32> = textureSample(t_normal, s_normal, in.tex_coords);

    // We don't need (or want) much ambient light, so 0.1 is fine
    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;

    let tangent_normal = object_normal.xyz * 2.0 - 1.0;
    let light_dir = normalize(light.position - in.world_position);
    let view_dir = normalize(camera.view_pos.xyz - in.world_position);
    // https://learnopengl.com/Advanced-Lighting/Advanced-Lighting
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(tangent_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * light.color;

    //let reflect_dir = reflect(-light_dir, in.world_normal);
    //let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);

    let result = (ambient_color + diffuse_color + specular_color) * object_color.xyz;
    //let result = ambient_color * object_color.xyz;
    //let result = diffuse_color * object_color.xyz;
    //let result = specular_color * object_color.xyz;

    return vec4<f32>(result, object_color.a);
}
