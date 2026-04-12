fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let lib_subdir = if cfg!(target_os = "linux") {
        "linux-x86_64"
    } else if cfg!(target_os = "macos") {
        "macos-arm64"
    } else if cfg!(target_os = "windows") {
        "windows-x86_64"
    } else {
        panic!("Unsupported platform for Vosk");
    };

    let lib_dir = format!("{}/.vosklib/{}", manifest_dir, lib_subdir);

    if !std::path::Path::new(&lib_dir).exists() {
        eprintln!(
            "ERROR: Vosk native library directory not found: {}",
            lib_dir
        );
        eprintln!("Run `mise run download-vosk-lib` to fetch the correct native library.");
        std::process::exit(1);
    }

    // vosk-sys already emits #[link(name = "vosk")] / #[link(name = "libvosk")]
    // via its #[link] attribute — only provide the search path here.
    println!("cargo:rustc-link-search=native={}", lib_dir);

    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir);
    } else if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir);
    }

    println!("cargo:rerun-if-changed=.vosklib/{}", lib_subdir);

    tauri_build::build()
}
