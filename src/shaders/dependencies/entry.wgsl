struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

fn raymarch(ray: Ray) -> vec4<f32> {
    var travel_distance = 0.;
    var position = ray.origin;
    for (var i = 0; i < options.max_iterations && travel_distance < options.max_distance; i++) {
        let distance = scene_SDF(position);

        if distance < options.epsilon {
            let normal = get_normal(position);
            let diffuse = 0.1 + 0.9 * clamp(dot(normal, vec3(1., 1., 1.)), 0., 1.);

            return vec4(diffuse * options.fractal_color, 1.);
        }

        travel_distance += distance;
        position = ray.origin + travel_distance * ray.direction;
    }

    return vec4(options.background_color, 1.);
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    @builtin(instance_index) in_instance_index: u32,
) -> VertexOutput {
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

    return raymarch(ray);
}
