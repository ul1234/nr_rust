#[macro_export]
macro_rules! into_variant {
    ($value:expr, $pattern:path) => {
        match &$value {
            $pattern(v) => v,
            _ => panic!("Pattern {:?} doesn't match!", stringify!($pattern)),
        }
    };
}

#[macro_export]
macro_rules! as_variant {
    ($value:expr, $pattern:path) => {
        match &$value {
            $pattern(v) => Some(v),
            _ => None,
        }
    };
}

#[macro_export]
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
