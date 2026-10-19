use std::process::Command;

/// systemd XDG autostart unit generated from rpci-ime.desktop.
pub const UNIT: &str = "app-rpci\\x2dime@autostart.service";

fn systemctl(action: &str) -> Result<(), String> {
    let status = Command::new("systemctl")
        .args(["--user", action, UNIT])
        .status()
        .map_err(|err| format!("failed to run systemctl: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("systemctl {action} exited with {status}"))
    }
}

/// Restart the installed input method through the systemd user manager.
pub fn restart_input_method() -> Result<(), String> {
    systemctl("restart")
}

/// Stop the installed input method before restoring its user data.
pub fn stop_input_method() -> Result<(), String> {
    systemctl("stop")
}

/// Start the installed input method after restoring its user data.
pub fn start_input_method() -> Result<(), String> {
    systemctl("start")
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
