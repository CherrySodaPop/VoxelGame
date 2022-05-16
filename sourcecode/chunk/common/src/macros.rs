/// Creates a `Vector2`, automatically converting parameters to `f32`.
#[macro_export]
macro_rules! vec2 {
    ($x:expr, $y:expr) => {
        Vector2::new($x as f32, $y as f32)
    };

    ($arr:expr) => {
        Vector2::new($arr[0] as f32, $arr[1] as f32)
    };
}

/// Creates a `Vector3`, automatically converting parameters to `f32`.
#[macro_export]
macro_rules! vec3 {
    ($x:expr, $y:expr, $z:expr) => {
        Vector3::new($x as f32, $y as f32, $z as f32)
    };

    ($arr:expr) => {
        Vector3::new($arr[0] as f32, $arr[1] as f32, $arr[2] as f32)
    };
}

/// Creates a `Color`. Parameters are expected to be in the range 0-255 and will be
/// converted to their corresponding f32 values automatically.
#[macro_export]
macro_rules! color {
    ($r:expr, $g:expr, $b: expr, $a: expr) => {
        Color::from_rgba(
            $r as f32 / 255.0,
            $g as f32 / 255.0,
            $b as f32 / 255.0,
            $a as f32 / 255.0,
        )
    };

    ($arr:expr) => {
        color!($arr[0], $arr[1], $arr[2], $arr[3])
    };
}
