use crate::shell::ShellTarget;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiskLevel {
    Safe,
    Risky,
    HighRisk,
    Blocked,
}

impl RiskLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Risky => "risky",
            Self::HighRisk => "high_risk",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CommandAssessment {
    pub level: RiskLevel,
    pub reasons: Vec<String>,
}

pub struct SafetyEngine;

impl SafetyEngine {
    pub fn assess(command: &str, shell: ShellTarget) -> CommandAssessment {
        let normalized = normalize(command);
        let mut level = RiskLevel::Safe;
        let mut reasons = Vec::new();

        if contains_any(&normalized, &blocked_patterns(shell)) {
            level = RiskLevel::Blocked;
            reasons.push("command matches an extreme destructive pattern".to_string());
        }

        if level != RiskLevel::Blocked && contains_any(&normalized, &high_risk_patterns(shell)) {
            level = RiskLevel::HighRisk;
            reasons.push("command may require elevated/system-level privileges".to_string());
        }

        if level == RiskLevel::Safe && contains_any(&normalized, &risky_patterns(shell)) {
            level = RiskLevel::Risky;
            reasons.push("command can modify files, install software, download content, or change local state".to_string());
        }

        if level == RiskLevel::Safe {
            reasons.push("read-only or low-impact command pattern".to_string());
        }

        CommandAssessment { level, reasons }
    }
}

fn normalize(command: &str) -> String {
    command
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn contains_any(command: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| command.contains(pattern))
}

fn blocked_patterns(shell: ShellTarget) -> Vec<&'static str> {
    let mut patterns = vec![
        "rm -rf /",
        "rm -fr /",
        "rm -rf /*",
        "mkfs",
        "dd if=",
        " of=/dev/",
        ":(){",
        "shutdown -h now",
        "reboot -f",
    ];

    if matches!(shell, ShellTarget::PowerShell | ShellTarget::Cmd) {
        patterns.extend([
            "format c:",
            "format /fs",
            "diskpart",
            "remove-item -recurse -force c:\\",
            "del /f /s /q c:\\",
            "rd /s /q c:\\",
        ]);
    }

    patterns
}

fn high_risk_patterns(shell: ShellTarget) -> Vec<&'static str> {
    let mut patterns = vec![
        "sudo ",
        "sudo\t",
        "su -",
        "doas ",
        "systemctl ",
        "service ",
        "launchctl ",
        "chmod ",
        "chown ",
        "/etc/",
        "/usr/bin/",
        "/usr/sbin/",
        "/bin/",
        "/sbin/",
        "/var/",
        "iptables ",
        "ufw ",
        "mount ",
        "umount ",
    ];

    if matches!(shell, ShellTarget::PowerShell | ShellTarget::Cmd) {
        patterns.extend([
            "start-process",
            "-verb runas",
            "set-itemproperty hklm:",
            "new-itemproperty hklm:",
            "remove-itemproperty hklm:",
            "reg add",
            "reg delete",
            "netsh ",
            "sc.exe ",
            "bcdedit",
            "takeown ",
            "icacls ",
            "restart-service",
            "stop-service",
            "start-service",
        ]);
    }

    patterns
}

fn risky_patterns(shell: ShellTarget) -> Vec<&'static str> {
    let mut patterns = vec![
        "rm ",
        "mv ",
        "cp ",
        "truncate ",
        "curl ",
        "wget ",
        "chmod ",
        "chown ",
        "apt install",
        "apt-get install",
        "brew install",
        "npm install",
        "pip install",
        "docker prune",
        "docker rm",
        "docker rmi",
        "git clean",
        "> ",
        ">> ",
        "tee ",
    ];

    if matches!(shell, ShellTarget::PowerShell | ShellTarget::Cmd) {
        patterns.extend([
            "remove-item",
            "move-item",
            "copy-item",
            "invoke-webrequest",
            "irm ",
            "iwr ",
            "del ",
            "erase ",
            "rmdir ",
            "rd ",
            "winget install",
            "choco install",
        ]);
    }

    patterns
}

#[cfg(test)]
mod tests {
    use super::{RiskLevel, SafetyEngine};
    use crate::shell::ShellTarget;

    #[test]
    fn blocks_root_delete() {
        let assessment = SafetyEngine::assess("rm -rf /", ShellTarget::Bash);
        assert_eq!(assessment.level, RiskLevel::Blocked);
    }

    #[test]
    fn detects_sudo_as_high_risk() {
        let assessment = SafetyEngine::assess("sudo apt update", ShellTarget::Bash);
        assert_eq!(assessment.level, RiskLevel::HighRisk);
    }

    #[test]
    fn keeps_find_safe() {
        let assessment = SafetyEngine::assess("find . -type f -name \"*.py\"", ShellTarget::Bash);
        assert_eq!(assessment.level, RiskLevel::Safe);
    }
}
