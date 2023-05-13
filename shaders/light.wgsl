struct Transform {
    matrix: mat4x4<f32>,
    ti_matrix: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> transform: Transform;

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: Camera;


struct PointLight {
    position: vec3<f32>,
    color: vec3<f32>,
};
@group(3) @binding(0)
var<storage, read> lights: array<PointLight>;
@group(3) @binding(1)
var<uniform> light_count: i32;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
};

@vertex
fn vs_main (
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera.view_proj * transform.matrix * vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;

    out.world_normal = normalize(transform.ti_matrix * vec4<f32>(model.normal, 0.0)).xyz;
    var world_position: vec4<f32> = transform.matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;

    return out;
}

@group(2) @binding(0)
var<uniform> diffuse_color: vec3<f32>;
@group(2) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(2)
var s_diffuse: sampler;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords) * vec4<f32>(diffuse_color, 1.0);

    let view_dir = normalize(camera.view_pos.xyz - in.world_position);

    var result = calculate_point_light(lights[0], in.world_position, in.world_normal, view_dir);

    for (var i: i32 = 1; i < light_count; i++) {
        result += calculate_point_light(lights[i], in.world_position, in.world_normal, view_dir);
    }

    result *= object_color.xyz;

    return vec4<f32>(result, object_color.a);
}

fn calculate_point_light(light: PointLight, world_position: vec3<f32>, world_normal: vec3<f32>, view_dir: vec3<f32>) -> vec3<f32>{
    let distance = length(light.position - world_position);
    let attenuation = 1.0 / (1.0 + 0.09 * distance + 0.032 * (distance * distance));

    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength * attenuation;

    let light_dir = normalize(light.position - world_position);
    let half_dir = normalize(view_dir + light_dir);

    let diffuse_strength = max(dot(world_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength * attenuation;

    let specular_strength = pow(max(dot(world_normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * light.color * attenuation;

    return (ambient_color + diffuse_color + specular_color);
}