const p = 3.;
const c = Quaternion(vec4(-0.1, 0.6, 0.9, -0.3));
const w = 0.1;
const JULIA_ITERATIONS = 100;
const JULIA_NORMAL_ITERATIONS = 10;

fn scene_SDF(position: vec3<f32>) -> f32 {
    // Approximation breaks when too far away, so we patch it non-continuously
    let norm = length(position);
    if norm > 2. + EPSILON {
        return norm - 2.;
    }

    var q = Quaternion(vec4(position, w));
    var q_sq_norm = quat_sq_norm2(q);
    var dq_sq_norm = 1.;
    for(var i = 0; i < JULIA_ITERATIONS; i++) {
        dq_sq_norm *= p * p * pow(q_sq_norm, p - 1.);
        q = quat_add(quat_pow(q, p), c);

        q_sq_norm = quat_sq_norm2(q);
        if(q_sq_norm > MAX_DISTANCE) {
            break;
        }
    }

    // Julia set distance approximation (for point close to the set) derived from Douady-Hubbard potential (Boettecher map)
    return 0.25 * log(q_sq_norm) * sqrt(q_sq_norm / dq_sq_norm);
}


fn get_normal(position: vec3<f32>) -> vec3<f32> {
    let h_x = vec3(EPSILON, 0., 0.);
    let h_y = vec3(0., EPSILON, 0.);
    let h_z = vec3(0., 0., EPSILON);

    var q_x_pos = Quaternion(vec4(position + h_x, w));
    var q_x_neg = Quaternion(vec4(position - h_x, w));
    var q_y_pos = Quaternion(vec4(position + h_y, w));
    var q_y_neg = Quaternion(vec4(position - h_y, w));
    var q_z_pos = Quaternion(vec4(position + h_z, w));
    var q_z_neg = Quaternion(vec4(position - h_z, w));
    for(var i = 0; i < JULIA_NORMAL_ITERATIONS; i++) {
        q_x_pos = quat_add(quat_pow(q_x_pos, p), c);
        q_x_neg = quat_add(quat_pow(q_x_neg, p), c);
        q_y_pos = quat_add(quat_pow(q_y_pos, p), c);
        q_y_neg = quat_add(quat_pow(q_y_neg, p), c);
        q_z_pos = quat_add(quat_pow(q_z_pos, p), c);
        q_z_neg = quat_add(quat_pow(q_z_neg, p), c);
    }

    return normalize(vec3(
        log2(quat_norm2(q_x_pos)) - log2(quat_norm2(q_x_neg)),
        log2(quat_norm2(q_y_pos)) - log2(quat_norm2(q_y_neg)),
        log2(quat_norm2(q_z_pos)) - log2(quat_norm2(q_z_neg)),
    ));
}
