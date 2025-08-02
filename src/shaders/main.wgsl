const MAX_ITERATIONS = 1000;
const MAX_DISTANCE = 1000.;
const EPSILON = 0.0001;

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

@group(0)
@binding(1)
var<uniform> camera: CameraUniform;

struct GuiUniform {
    fractal_color: vec4<f32>,
    background_color: vec4<f32>,
    primitive_id: u32,
}

@group(0)
@binding(2)
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

    out.clip_position = vec4(x, y, 0., 1.);
    return out;
}

fn scene_SDF(position: vec3<f32>) -> f32 {
    if options.primitive_id == 0 {
        let sphere = Sphere(1.);
        return sphere_SDF(sphere, position);
    } else if options.primitive_id == 1 {
        let cylinder = Cylinder(1., 2.);
        return cylinder_SDF(cylinder, position);
    } else if options.primitive_id == 2 {
        let box = Box(1., 1., 1.);
        return box_SDF(box, position);
    } else if options.primitive_id == 3 {
        let torus = Torus(1., 0.3);
        return torus_SDF(torus, position);
    } else if options.primitive_id == 4 {
        return sierpinski_tetrahedron_SDF(position);
    } else if options.primitive_id == 5 {
        return bunny_SDF(position);
    }

    return 1.;
}

fn get_normal(position: vec3<f32>) -> vec3<f32> {
    let h = EPSILON;
    
    let h_x = vec3(h, 0., 0.);
    let h_y = vec3(0., h, 0.);
    let h_z = vec3(0., 0., h);

    let sdf_dx = scene_SDF(position + h_x) - scene_SDF(position - h_x);
    let sdf_dy = scene_SDF(position + h_y) - scene_SDF(position - h_y);
    let sdf_dz = scene_SDF(position + h_z) - scene_SDF(position - h_z);

    return normalize(vec3(sdf_dx, sdf_dy, sdf_dz));
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct CollisionPoint {
    valid_hit: bool,
    color: vec4<f32>,
    travel_distance: f32,
}

fn raymarch(ray: Ray) -> CollisionPoint {
    var travel_distance = 0.;

    var position = ray.origin;
    for (var i = 0; i < MAX_ITERATIONS && travel_distance < MAX_DISTANCE; i++) {
        let distance = scene_SDF(position);

        if distance < EPSILON {
            let normal = get_normal(position);
            let diffuse = 0.1 + 0.9 * max(dot(normal, vec3(1., 1., 1.)), 0.);

            return CollisionPoint(true, vec4(diffuse * options.fractal_color.xyz, options.fractal_color.w), travel_distance);
        }

        travel_distance += distance;
        position = ray.origin + travel_distance * ray.direction;
    }

    return CollisionPoint(false, options.background_color, travel_distance);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let camera_matrix = camera.matrix;
    let origin = camera.origin_distance * camera_matrix * vec3(1., 0., 0.);

    let translated_position = in.clip_position.xy - 0.5 * vec2(f32(size.width), f32(size.height));
    let uv_position = 2. * translated_position / f32(size.height);
    let direction = normalize(camera_matrix * vec3(-1., uv_position.x, -uv_position.y));

    let ray = Ray(origin, direction);

    return raymarch(ray).color;
}