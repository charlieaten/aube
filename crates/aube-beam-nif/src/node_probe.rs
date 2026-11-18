use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

pub(crate) fn version(program: &Path) -> Option<String> {
    // OTP can reap this process's children on macOS. Relay Node's output
    // only on success so the version is valid even when BEAM owns the status.
    let mut child = Command::new("/bin/sh")
        .args([
            "-c",
            r#"version=$("$1" --version) && printf '%s\n' "$version""#,
            "aube-node-probe",
        ])
        .arg(program)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut version = String::new();
    let output = child.stdout.take()?.read_to_string(&mut version);

    // A blocking wait can miss a child exit when the BEAM reaper wins.
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => break,
            Ok(None) => std::thread::sleep(Duration::from_millis(1)),
            Err(error) if error.raw_os_error() == Some(libc::ECHILD) => break,
            _ => return None,
        }
    }

    output.ok()?;
    if version.trim().is_empty() {
        return None;
    }
    Some(version)
}

#[cfg(test)]
mod tests {
    use super::version;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn probes_executable_paths_and_rejects_output_from_a_failed_probe() {
        let temp = tempfile::tempdir().expect("create probe fixture directory");
        let node = temp.path().join("node with spaces");
        fs::write(&node, "#!/bin/sh\nprintf 'v26.13.1\\n'\n").expect("write probe fixture");
        fs::set_permissions(&node, fs::Permissions::from_mode(0o700))
            .expect("mark probe fixture executable");
        assert_eq!(version(&node).as_deref(), Some("v26.13.1\n"));

        fs::write(&node, "#!/bin/sh\nprintf 'v26.13.1\\n'\nexit 42\n")
            .expect("write failing probe fixture");
        assert_eq!(version(&node), None);
    }
}
