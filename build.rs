use std::fs;
use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let profile = env::var("PROFILE").unwrap_or("debug".to_string());
    let current_dir = env::current_dir().unwrap();

    let target;
    if profile == "release" {
        target = Path::new(&current_dir).join("target/release");
    } else {
        target = Path::new(&current_dir).join("target/debug");
    }

    fs::remove_dir(Path::new(&target).join("reloaded"))
        .unwrap_or_else(|e| println!("Failed to clean reloaded directory: {}", e));

    Command::new("rustc")
        .arg("waved-core/src/lib.rs")
        .arg("--crate-name")
        .arg("waved_core")
        .arg("--crate-type")
        .arg("dylib")
        .arg("--out-dir")
        .arg(target)
        .output()
        .unwrap_or_else(|e| panic!("Failed to execute process: {}", e));
}
