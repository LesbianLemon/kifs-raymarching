struct ScreenUniform {
    width: f32,
    height: f32,
    aspect_ratio: f32,
}

@group(0)
@binding(0)
var<uniform> screen: ScreenUniform;

struct CameraUniform {
    origin: vec3<f32>,
    matrix: mat3x3<f32>,
}

@group(0)
@binding(1)
var<uniform> camera: CameraUniform;

struct OptionsUniform {
    max_iterations: i32,
    max_distance: f32,
    epsilon: f32,
    fractal_color: vec3<f32>,
    background_color: vec3<f32>,
    fractal_group_id: u32,
    primitive_id: u32,
    power: f32,
    constant: Quaternion,
}

@group(0)
@binding(2)
var<uniform> options: OptionsUniform;
