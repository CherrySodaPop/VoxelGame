/// Creates a `Vector2`, automatically converting parameters to `f32`.
macro_rules! vec2 {
    ($x:expr, $y:expr) => {
        Vector2::new($x as f32, $y as f32)
    };
}

/// Creates a `Vector3`, automatically converting parameters to `f32`.
macro_rules! vec3 {
    ($x:expr, $y:expr, $z:expr) => {
        Vector3::new($x as f32, $y as f32, $z as f32)
    };
}

pub(crate) use vec2;
pub(crate) use vec3;
