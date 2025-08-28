use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn main() {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=../flutter_rust_bridge.yaml");

    if std::env::var("SKIP_CODEGEN").is_ok() {
        println!("cargo:warning=Skipping code generation (SKIP_CODEGEN is set)");
        return;
    }

    let check_cli = Command::new("flutter_rust_bridge_codegen")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if check_cli.is_err() || !check_cli.unwrap().success() {
        println!("cargo:warning=flutter_rust_bridge_codegen not available, skipping");
        return;
    }

    println!("cargo:warning=Starting code generation with 30s timeout...");

    let handle = thread::spawn(|| {
        Command::new("flutter_rust_bridge_codegen")
            .args(&["generate"])
            .current_dir("..")
            .output()
    });

    thread::sleep(Duration::from_secs(30));

    if handle.is_finished() {
        match handle.join() {
            Ok(Ok(output)) => {
                if output.status.success() {
                    println!("cargo:warning=Code generation completed successfully");
                } else {
                    println!("cargo:warning=Code generation failed");
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("cargo:warning=Error: {}", stderr);
                }
            }
            Ok(Err(e)) => {
                println!("cargo:warning=Failed to execute: {}", e);
            }
            Err(_) => {
                println!("cargo:warning=Code generation thread panicked");
            }
        }
    } else {
        println!("cargo:warning=Code generation timed out after 30 seconds");
        println!("cargo:warning=Try running manually: flutter_rust_bridge_codegen generate");
    }
}
