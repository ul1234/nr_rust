#[macro_export]
macro_rules! enum_content {
    ($value:expr, $pattern:path) => {
        match &$value {
            $pattern(v) => v,
            _ => panic!("Pattern {:?} doesn't match!", stringify!($pattern)),
        }
    };
}
