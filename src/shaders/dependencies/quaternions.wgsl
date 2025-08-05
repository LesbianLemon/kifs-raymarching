struct Quaternion {
    // Inner vec4<f32> representing vec4(real, i, j, k)
    inner: vec4<f32>,
}

fn quat_as_vec(q: Quaternion) -> vec4<f32> {
    return q.inner;
}

fn quat_real(q: Quaternion) -> f32 {
    return q.inner.x;
}

fn quat_ijk(q: Quaternion) -> vec3<f32> {
    return q.inner.yzw;
}

fn quat_norm2(q: Quaternion) -> f32 {
    return length(q.inner);
}

fn quat_sq_norm2(q: Quaternion) -> f32 {
    return dot(q.inner, q.inner);
}

fn quat_add(lhs: Quaternion, rhs: Quaternion) -> Quaternion {
    return Quaternion(lhs.inner + rhs.inner);
}

fn quat_mul(lhs: Quaternion, rhs: Quaternion) -> Quaternion {
    let lhs_real = quat_real(lhs);
    let lhs_ijk = quat_ijk(lhs);
    let rhs_real = quat_real(rhs);
    let rhs_ijk = quat_ijk(rhs);

    return Quaternion(vec4(
        lhs_real * rhs_real - dot(lhs_ijk, rhs_ijk),
        lhs_real * rhs_ijk + rhs_real * lhs_ijk + cross(lhs_ijk, rhs_ijk),
    ));
}

fn quat_sq(q: Quaternion) -> Quaternion {
    let q_real = quat_real(q);
    let q_ijk = quat_ijk(q);

    return Quaternion(vec4(
        q_real * q_real - dot(q_ijk, q_ijk),
        2.0 * q_real * q_ijk,
    ));
}

fn quat_scalar_mul(scalar: f32, q: Quaternion) -> Quaternion {
    return Quaternion(scalar * q.inner);
}

// Source: https://en.wikipedia.org/wiki/Quaternion#Exponential,_logarithm,_and_power_functions
fn quat_pow(q: Quaternion, x: f32) -> Quaternion {
    let norm = quat_norm2(q);
    let phi = acos(quat_real(q) / norm);
    let n = normalize(quat_ijk(q));

    return Quaternion(pow(norm, x) * vec4(cos(x * phi), n * sin(x * phi)));
}
