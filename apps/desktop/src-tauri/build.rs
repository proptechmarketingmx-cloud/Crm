fn main() {
    // Tauri validates frontendDist during cargo check, before npm build has run.
    std::fs::create_dir_all("../build").expect("create frontendDist directory");
    tauri_build::build()
}
