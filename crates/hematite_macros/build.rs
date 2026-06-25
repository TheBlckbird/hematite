use std::{
    env::{self, current_dir},
    error::Error,
    fs,
    path::PathBuf,
    process::Command,
};

/// Minecraft Server JAR download link for version 1.21.10
///
/// See https://gist.github.com/cliffano/77a982a7503669c3e1acb0a0cf6127e9
const SERVER_URL: &str =
    "https://piston-data.mojang.com/v1/objects/95495a7f485eedd84ce928cef5e223b757d2f764/server.jar";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let packets_json_path = current_dir()?
        .join("src")
        .join("packet")
        .join("packets.json");

    if packets_json_path.exists() {
        return Ok(());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let destination = out_dir.join("minecraft_server.jar");

    let curl_status = Command::new("curl")
        .args(["-fsSL", "-o", destination.to_str().unwrap(), SERVER_URL])
        .status()
        .expect("failed to run curl");

    assert!(
        curl_status.success(),
        "Failed to download server jar. Is `curl` installed?"
    );

    let datagen_status = Command::new("java")
        .args([
            "-DbundlerMainClass=net.minecraft.data.Main",
            "-jar",
            destination.to_str().unwrap(),
            "--reports",
        ])
        .current_dir(destination.parent().unwrap())
        .status()
        .expect("failed to run java");

    assert!(
        datagen_status.success(),
        "Failed to run datagen for Minecraft. Is `java` version 17 installed?"
    );

    fs::rename(
        out_dir
            .join("generated")
            .join("reports")
            .join("packets.json"),
        packets_json_path,
    )
    .expect("Failed to move packets.json file");

    Ok(())
}
