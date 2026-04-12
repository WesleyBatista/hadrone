use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = PathBuf::from(manifest_dir)
        .parent()
        .expect("examples crate should live under workspace root")
        .to_path_buf();
    let wasm_crate_dir = workspace_root.join("crates").join("hadrone-wasm");
    let example_dir = PathBuf::from(manifest_dir).join("vanilla-js");

    // 1. Check if wasm-pack is installed
    if Command::new("wasm-pack").arg("--version").output().is_err() {
        eprintln!("Error: `wasm-pack` is not installed.");
        eprintln!("Install it with: cargo install wasm-pack");
        std::process::exit(1);
    }

    println!("Building WASM bindings for Vanilla JS...");
    let status = Command::new("wasm-pack")
        .args(&[
            "build",
            "--target",
            "web",
            "--out-dir",
            "../../examples/vanilla-js/pkg",
        ])
        .current_dir(&wasm_crate_dir)
        .status()
        .expect("Failed to execute wasm-pack");

    if !status.success() {
        eprintln!("wasm-pack build failed!");
        std::process::exit(1);
    }

    println!("Build successful!");
    println!("The Vanilla JS example is located at: {:?}", example_dir);
    println!("\nTo view the example, you need a local web server.");
    println!("You can run:");
    println!("  cd examples/vanilla-js && python3 -m http.server 8083");
    println!("\nThen open: http://localhost:8083");

    // Try to auto-start python server if possible (optional)
    println!("\nAttempting to start local server...");
    let _ = Command::new("python3")
        .args(&["-m", "http.server", "8083"])
        .current_dir(&example_dir)
        .status();
}
