struct SizeUniform {
    width: u32,
    height: u32,
}

@group(0)
@binding(0)
var<uniform> size: SizeUniform;

struct CameraUniform {
    origin_distance: f32,
    min_distance: f32,
    angles: vec2<f32>,
    matrix: mat3x3<f32>,
}

@group(1)
@binding(0)
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    let x = f32(1 - i32(in_vertex_index)) * 3.;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 3.;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    return out;
}

struct Sphere {
    center: vec3<f32>,
    radius: f32,
}

fn sphere_SDF(sphere: Sphere, position: vec3<f32>) -> f32 {
    return distance(sphere.center, position) - sphere.radius;
}

fn axes_SDF(thickness: f32, position: vec3<f32>) -> f32 {
    return min(length(position.yz), min(length(position.xz), length(position.xy))) - thickness;
}

fn scene_SDF(position: vec3<f32>) -> f32 {
    let sphere = Sphere(vec3<f32>(0., 0., 0.), 5.);

    return sphere_SDF(sphere, position);
}

fn get_normal(position: vec3<f32>) -> vec3<f32> {
    let h = 0.0001;
    
    let h_x = vec3<f32>(h, 0., 0.);
    let h_y = vec3<f32>(0., h, 0.);
    let h_z = vec3<f32>(0., 0., h);

    let sdf_dx = scene_SDF(position + h_x) - scene_SDF(position - h_x);
    let sdf_dy = scene_SDF(position + h_y) - scene_SDF(position - h_y);
    let sdf_dz = scene_SDF(position + h_z) - scene_SDF(position - h_z);

    return normalize(vec3<f32>(sdf_dx, sdf_dy, sdf_dz));
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let camera_matrix = camera.matrix;
    let origin = camera.origin_distance * camera_matrix * vec3<f32>(1., 0., 0.);

    let translated_position = in.clip_position.xy - 0.5 * vec2<f32>(f32(size.width), f32(size.height));
    let uv_position = 2. * translated_position / f32(size.height);
    let direction = normalize(camera_matrix * vec3<f32>(-1., uv_position.x, -uv_position.y));

    var origin_distance: f32;
    var position = origin;
    for (var i = 0; i < 100 && origin_distance < 1000.; i++) {
        let scene_distance = scene_SDF(position);
        let axis_distance = axes_SDF(0.05, position);
        let distance = min(scene_distance, axis_distance);

        if scene_distance < 0.001 {
            let normal = get_normal(position);
            let diffuse = 0.1 + 0.9 * max(dot(normal, vec3<f32>(1., 1., 1.)), 0.);

            return vec4<f32>(diffuse, diffuse, diffuse, 1.);
        }

        if axis_distance < 0.001 {
            return vec4<f32>(0.5, 0.5, 0.5, 0.25);
        }

        origin_distance += distance;
        position += distance*direction;
    }

    return vec4<f32>(0., 0., 0., 1.);
}