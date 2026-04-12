use std::path::PathBuf;
use std::process::Command;

fn main() {
    // The actual WASM app lives in examples/leptos-dashboard/
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let example_dir = PathBuf::from(manifest_dir).join("leptos-dashboard");

    // Check if trunk is installed
    if Command::new("trunk").arg("--version").output().is_err() {
        eprintln!("Error: `trunk` is not installed.");
        eprintln!("Install it with: cargo install trunk");
        std::process::exit(1);
    }

    println!("Starting Leptos dashboard...");
    println!("Open http://localhost:8081 in your browser");

    let status = Command::new("trunk")
        .args(&["serve", "--port", "8081", "--open"])
        .current_dir(&example_dir)
        .status()
        .expect("Failed to execute trunk");

    if !status.success() {
        eprintln!("trunk serve failed!");
        std::process::exit(1);
    }
}
