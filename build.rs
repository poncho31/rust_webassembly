use std::process::Command;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=../client/src");
    println!("cargo:rerun-if-changed=../client/Cargo.toml");
    
    let workspace_dir = env::current_dir().unwrap().parent().unwrap().to_path_buf();
    let client_dir = workspace_dir.join("client");
    
    // Nettoyage et création du dossier pkg
    let pkg_dir = client_dir.join("static").join("pkg");
    if pkg_dir.exists() {
        std::fs::remove_dir_all(&pkg_dir).unwrap_or_else(|e| {
            println!("cargo:warning=Failed to clean pkg directory: {}", e);
        });
    }
    
    std::fs::create_dir_all(&pkg_dir).unwrap_or_else(|e| {
        println!("cargo:warning=Failed to create pkg directory: {}", e);
    });

    println!("cargo:warning=Building WebAssembly package...");
    
    let status = Command::new("wasm-pack")
        .current_dir(&client_dir)
        .args(&[
            "build",
            "--target", "web",
            "--out-dir", "static/pkg",
            "--verbose"
        ])
        .status()
        .unwrap_or_else(|e| {
            println!("cargo:warning=Failed to execute wasm-pack: {}", e);
            std::process::exit(1);
        });

    if !status.success() {
        println!("cargo:warning=wasm-pack build failed");
        std::process::exit(1);
    }

    println!("cargo:warning=WebAssembly build completed successfully!");
}
