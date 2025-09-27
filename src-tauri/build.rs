use base64::{engine::general_purpose, Engine as _};
use std::{env, fs, path::PathBuf};

fn main() {
    // Ensure a valid RGBA icon exists so tauri::generate_context!() doesn't panic
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let icons_dir = PathBuf::from(&manifest_dir).join("icons");
    let icon_path = icons_dir.join("icon.png");
    let _ = fs::create_dir_all(&icons_dir);
    // 1x1 transparent RGBA PNG
    let png_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAGgwJ/lToYVAAAAABJRU5ErkJggg==";
    if let Ok(bytes) = general_purpose::STANDARD.decode(png_base64) {
        let _ = fs::write(&icon_path, bytes);
        println!("cargo:rerun-if-changed={}", icon_path.display());
    }

    tauri_build::build()
}
