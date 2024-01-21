use std::path::Path;

use colored::Colorize;

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

pub fn nix_shell(packages: impl AsRef<[&'static str]>) -> Option<std::process::Command> {
    let packages = packages.as_ref();
    let nix = get_nix()?;
    
    if is_flakes(&nix) {
        println!("{}", "Using flakes:".dimmed());
        let mut cmd = std::process::Command::new(nix);
        cmd.arg("shell")
            .args(packages.iter().map(|p| format!("nixpkgs#{p}")))
            .arg("--command");
        Some(cmd)
    } else {
        println!("{}", "Using nix-shell:".dimmed());
        let mut cmd = std::process::Command::new(get_nix_shell()?);
        cmd.arg("--packages").arg(packages.join(" ")).arg("--run");
        Some(cmd)
    }
}
