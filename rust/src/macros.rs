macro_rules! debug_print {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        godot::prelude::godot_print!($($arg)*);
    };
}
