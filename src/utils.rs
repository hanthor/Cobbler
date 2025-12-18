use anyhow::{Context, Result};
use std::process::Command;
use log::info;
use std::path::Path;

pub fn is_flatpak() -> bool {
    Path::new("/.flatpak-info").exists()
}

pub fn run_command(cmd: &str, args: &[&str]) -> Result<()> {
    info!("Running: {} {:?}", cmd, args);
    let status = Command::new(cmd)
        .args(args)
        .status()
        .context(format!("Failed to run {}", cmd))?;

    if !status.success() {
        anyhow::bail!("Command {} {:?} failed with {}", cmd, args, status);
    }
    Ok(())
}

pub fn run_privileged_command(program: &str, args: &[&str]) -> Result<()> {
    let mut cmd_process = if is_flatpak() {
        let mut c = Command::new("flatpak-spawn");
        c.arg("--host");
        c.arg("pkexec");
        c
    } else {
        Command::new("pkexec")
    };

    cmd_process.arg(program);
    cmd_process.args(args);

    let cmd_string = format!("(privileged) {} {:?}", program, args);
    info!("Running: {}", cmd_string);

    let status = cmd_process
        .status()
        .context(format!("Failed to run privileged command: {}", program))?;

    if !status.success() {
        anyhow::bail!("Privileged command failed with {}", status);
    }
    Ok(())
}
