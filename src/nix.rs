use std::path::Path;
use yansi::Paint as _;

pub fn is_nix() -> bool {
    get_nix().is_some()
}

pub fn get_nix() -> Option<std::path::PathBuf> {
    which::which("nix").ok()
}

pub fn get_nix_shell() -> Option<std::path::PathBuf> {
    which::which("nix-shell").ok()
}

pub fn is_flakes(nix: &Path) -> bool {
    std::process::Command::new(nix)
        .arg("shell")
        .output()
        .ok()
        .is_some()
}

pub fn is_in_store(packages: impl AsRef<[&'static str]>) -> bool {
    let mut packages = packages.as_ref().to_vec();
    for package in std::fs::read_dir("/nix/store")
        .expect("Could not read /nix/store")
        .flatten()
    {
        let name = package.file_name();
        packages.retain(|p| !name.to_string_lossy().contains(p));

        if packages.is_empty() {
            return true;
        }
    }

    false
}

pub fn nix_shell(
    packages: impl AsRef<[&'static str]>,
    executable: &'static str,
) -> Option<std::process::Command> {
    let packages = packages.as_ref();
    let nix = get_nix()?;
    let first_time = !is_in_store(packages);

    if is_flakes(&nix) {
        if first_time {
            eprintln!(
                "{}",
                "Using flakes (could take a while if it's the first time):"
                    .dim()
                    .bright_black()
            );
        }
        let mut cmd = std::process::Command::new(nix);
        cmd.arg("shell")
            .args(packages.iter().map(|p| format!("nixpkgs#{p}")))
            .arg("--command")
            .arg(executable);
        Some(cmd)
    } else {
        if first_time {
            eprintln!(
                "{}",
                "Using nix-shell (could take a while if it's the first time):".dim()
            );
        }
        let mut cmd = std::process::Command::new(get_nix_shell()?);
        cmd.arg("--packages")
            .arg(packages.join(" "))
            .arg("--run")
            .arg(executable);
        Some(cmd)
    }
}
