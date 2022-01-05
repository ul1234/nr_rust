#[macro_export]
macro_rules! enum_content {
    ($value:expr, $pattern:path) => {
        match &$value {
            $pattern(v) => v,
            _ => panic!("Pattern {:?} doesn't match!", stringify!($pattern)),
        }
    };
}

macro_rules! check_func {
    ($name:ident, $pattern:pat) => {
        fn $name(&self) -> bool {
            match &self {
                $pattern => true,
                _ => false,
            }
        }
    };
    ($name:ident, $pattern:pat, $($pattern2:pat),+) => {
        fn $name(&self) -> bool {
            match &self {
                $pattern $(| $pattern2)+ => true,
                _ => false,
            }
        }
    };
}
