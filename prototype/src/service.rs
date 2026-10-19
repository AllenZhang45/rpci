use std::process::Command;

/// systemd XDG autostart unit generated from rpci-ime.desktop.
pub const UNIT: &str = "app-rpci\\x2dime@autostart.service";

/// Restart the installed input method through the systemd user manager.
pub fn restart_input_method() -> Result<(), String> {
    let status = Command::new("systemctl")
        .args(["--user", "restart", UNIT])
        .status()
        .map_err(|err| format!("failed to run systemctl: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("systemctl exited with {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_name_uses_systemd_escape() {
        assert_eq!(UNIT, "app-rpci\\x2dime@autostart.service");
        assert!(UNIT.contains("\\x2d"));
    }
}
