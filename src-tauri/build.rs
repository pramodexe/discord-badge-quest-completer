use std::path::PathBuf;

fn main() {
    tauri_build::build();

    // The stand-in program is built into the app (see RUNNER in src/lib.rs),
    // so it has to exist before the app is compiled.
    let runner = match std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default().as_str() {
        "windows" => "src-win.exe",
        "linux" => "src-linux",
        "macos" => "src-darwin",
        _ => return,
    };
    let path = PathBuf::from("resources").join(runner);
    println!("cargo:rerun-if-changed={}", path.display());
    if !path.exists() {
        panic!(
            "\n\nThe stand-in program {} is missing.\nRun `pnpm sync:runner` first, then build again.\n\n",
            path.display()
        );
    }
}
