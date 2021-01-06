use std::{fs::canonicalize, env::set_current_dir, path::Path, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=../client");
    println!("cargo:rerun-if-changed=static");

    let server_dir = canonicalize(Path::new(".")).expect("Failed to resolve path");
    let client_dir = Path::new("../client");

    set_current_dir(client_dir).expect("Failed to change to client directory");
    Command::new("yarn").arg("install").status().expect("Failed to install dependencies");
    Command::new("yarn").arg("build").status().expect("Failed to build app");
    set_current_dir(server_dir).expect("Failed to change back to server directory");
    Command::new("cp").args(&["-r", "../client/build/", "./static"]).status().expect("Failed to copy build output");
}
