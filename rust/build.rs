fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").map_or(false, |v| v == "windows") {
        println!("cargo:rustc-link-lib=wevtapi");
    }
}
