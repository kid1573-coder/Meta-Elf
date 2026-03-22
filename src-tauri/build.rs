fn main() {
    // Icons are embedded at link time; without this, changing `icons/*` may not rerun the build script
    // and the exe/taskbar icon stays stale until a Rust source file changes.
    println!("cargo:rerun-if-changed=icons");
    println!("cargo:rerun-if-changed=tauri.conf.json");
    tauri_build::build()
}
