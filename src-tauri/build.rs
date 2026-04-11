fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let vosklib_path = format!("{}/.vosklib", manifest_dir);
    println!("cargo:rustc-link-search=native={}", vosklib_path);
    println!("cargo:rustc-link-lib=vosk");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", vosklib_path);
    println!("cargo:rerun-if-changed=.vosklib");
    println!("cargo:rerun-if-changed=.vosklib/libvosk.so");

    tauri_build::build()
}
