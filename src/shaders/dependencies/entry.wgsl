const MAX_ITERATIONS = 256;
const MAX_DISTANCE = 1000.;
const EPSILON = 0.0001;

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct Collision {
    color: vec4<f32>,
    travel_distance: f32,
}

fn raymarch(ray: Ray) -> Collision {
    var travel_distance = 0.;
    var position = ray.origin;
    for (var i = 0; i < MAX_ITERATIONS && travel_distance < MAX_DISTANCE; i++) {
        let distance = scene_SDF(position);

        if distance < EPSILON {
            let diffuse = 0.1 + 0.9 * max(dot(get_normal(position), vec3(1., 1., 1.)), 0.);

            return Collision(vec4(diffuse * options.fractal_color.xyz, options.fractal_color.w), travel_distance);
        }

        travel_distance += distance;
        position = ray.origin + travel_distance * ray.direction;
    }

    return Collision(options.background_color, travel_distance);
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    @builtin(instance_index) in_instance_index: u32,
) -> VertexOutput {
    // var out: VertexOutput;

    // let x = f32(1 - i32(in_vertex_index)) * 3.;
    // let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 3.;

    // out.position = vec4(x, y, 0., 1.);
    // out.pixel_position = vec2(x, y);
    // return out;

    var out: VertexOutput;

    let x = f32((in_vertex_index & 1u) ^ in_instance_index);
    let y = f32((in_vertex_index >> 1u) ^ in_instance_index);

    out.position = vec4<f32>(2. * vec2(x, y) - 1., 0., 1.);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv_position = 2. * in.position.xy / screen.height - vec2(screen.aspect_ratio, 1.);

    // Matrix columns are the transformed vector basis, we use those to find pixel positions in space
    // Note: uv_position.y is flipped (-1 at top of screen and 1 at bottom)
    let ray_direction = normalize(uv_position.x * camera.matrix[1] - uv_position.y * camera.matrix[2] - camera.matrix[0]);
    let ray = Ray(camera.origin, ray_direction);

    return raymarch(ray).color;
}
