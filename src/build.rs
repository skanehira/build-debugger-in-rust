use std::{error::Error, process::Command};

pub fn build_debuggee_program(bin: &str) -> Result<(), Box<dyn Error>> {
    let output = Command::new("cargo")
        .args(["build", "--bin", bin])
        .output()?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Command failed: {}", error).into());
    }

    Ok(())
}

