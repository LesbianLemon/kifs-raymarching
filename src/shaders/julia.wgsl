const w = 0.1;
const JULIA_ITERATIONS = 100;
const JULIA_NORMAL_ITERATIONS = 10;

fn scene_SDF(position: vec3<f32>) -> f32 {
    // Approximation breaks when too far away, so we patch it non-continuously
    let norm = length(position);
    if norm > 2. + options.epsilon {
        return norm - 2.;
    }

    var q = Quaternion(vec4(position, w));
    var q_sq_norm = quat_sq_norm2(q);
    var dq_sq_norm = 1.;
	for(var i = 0; i < JULIA_ITERATIONS; i++) {
		dq_sq_norm *= 4. * q_sq_norm;
        q = quat_add(quat_sq(q), options.constant);

        q_sq_norm = quat_sq_norm2(q);
        if(q_sq_norm > options.max_distance) {
            break;
        }
	}

	// Julia set distance approximation (for point close to the set) derived from Douady-Hubbard potential (Boettecher map)
	return 0.25 * log(q_sq_norm) * sqrt(q_sq_norm / dq_sq_norm);;
}

fn get_normal(position: vec3<f32>) -> vec3<f32> {
    var q_vec = vec4(position, w);
	var q = Quaternion(q_vec);
    // Jacobian matrix
    var J = mat4x4(
        1., 0., 0., 0.,
        0., 1., 0., 0.,
        0., 0., 1., 0.,
        0., 0., 0., 1.,
    );
  	for(var i = 0; i < JULIA_NORMAL_ITERATIONS; i++) {
        J = mat4x4(
            q_vec.x, -q_vec.y, -q_vec.z, -q_vec.w,
            q_vec.y,  q_vec.x,       0.,       0.,
            q_vec.z,       0.,  q_vec.x,       0.,
            q_vec.w,       0.,       0.,  q_vec.x,
        ) * J;

        q = quat_add(quat_sq(q), options.constant); 
        q_vec = quat_as_vec(q);
        
        if(quat_sq_norm2(q) > options.max_distance) {
			break;
		}
    }
    
    return normalize((J * q_vec).xyz);
}
