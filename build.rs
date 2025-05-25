use std::process::Command;
use std::env;

fn main() {
    // Obtenir le chemin du répertoire courant
    let current_dir = env::current_dir().unwrap();
    let client_dir = current_dir.join("client");

    // Créer le dossier static/pkg s'il n'existe pas
    let pkg_dir = client_dir.join("static").join("pkg");
    std::fs::create_dir_all(&pkg_dir).unwrap();

    // Exécuter wasm-pack
    let status = Command::new("wasm-pack")
        .current_dir(&client_dir)
        .args(&["build", "--target", "web", "--out-dir", "static/pkg"])
        .status()
        .expect("Failed to execute wasm-pack");

    if !status.success() {
        panic!("wasm-pack build failed");
    }
}
