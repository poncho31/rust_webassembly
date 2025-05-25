use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=client/src");
    
    let status = Command::new("wasm-pack")
        .args(["build", "--target", "web", "--out-dir", "static/pkg"])
        .current_dir("client")
        .status()
        .expect("Failed to execute wasm-pack");

    if !status.success() {
        panic!("wasm-pack build failed");
    }
}
