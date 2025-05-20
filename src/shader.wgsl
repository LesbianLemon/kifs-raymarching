const MAX_ITERATIONS = 100;
const MAX_DISTANCE = 1000.;

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

struct GuiUniform {
    color: vec4<f32>,
}

@group(2)
@binding(0)
var<uniform> options: GuiUniform;

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

struct CollisionPoint {
    valid: bool,
    color: vec4<f32>,
    camera_distance: f32,
}

fn raymarch_scene(ray: Ray) -> CollisionPoint {
    var camera_distance = 0.;

    var position = ray.origin;
    for (var i = 0; i < MAX_ITERATIONS && camera_distance < MAX_DISTANCE; i++) {
        let distance = scene_SDF(position);

        if distance < 0.001 {
            let normal = get_normal(position);
            let diffuse = 0.1 + 0.9 * max(dot(normal, vec3<f32>(1., 1., 1.)), 0.);

            return CollisionPoint(true, vec4<f32>(diffuse, diffuse, diffuse, 1.), camera_distance);
        }

        camera_distance += distance;
        position += distance * ray.direction;
    }

    return CollisionPoint(false, options.color, camera_distance);
}

fn raymarch_axes(ray: Ray) -> CollisionPoint {
    var camera_distance = 0.;

    var position = ray.origin;
    for (var i = 0; i < MAX_ITERATIONS && camera_distance < MAX_DISTANCE; i++) {
        let distance = axes_SDF(0.1, position);

        if distance < 0.001 {
            return CollisionPoint(true, vec4<f32>(0.9, 0.9, 0.9, 1.), camera_distance);
        }

        camera_distance += distance;
        position += distance * ray.direction;
    }

    return CollisionPoint(false, options.color, camera_distance);
}

fn compare_distances(collision1: CollisionPoint, collision2: CollisionPoint) -> vec4<f32> {
    if collision1.camera_distance <= collision2.camera_distance {
        return collision1.color;
    } else {
        return collision2.color;
    }
}

fn compare_collisions(collision1: CollisionPoint, collision2: CollisionPoint) -> vec4<f32> {
    if collision1.valid && collision2.valid {
        return compare_distances(collision1, collision2);
    } else if collision1.valid && !collision2.valid {
        return collision1.color;
    } else if !collision1.valid && collision2.valid {
        return collision2.color;
    } else {
        return compare_distances(collision1, collision2);
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let camera_matrix = camera.matrix;
    let origin = camera.origin_distance * camera_matrix * vec3<f32>(1., 0., 0.);

    let translated_position = in.clip_position.xy - 0.5 * vec2<f32>(f32(size.width), f32(size.height));
    let uv_position = 2. * translated_position / f32(size.height);
    let direction = normalize(camera_matrix * vec3<f32>(-1., uv_position.x, -uv_position.y));

    let ray = Ray(origin, direction);

    return compare_collisions(raymarch_scene(ray), raymarch_axes(ray));    
}