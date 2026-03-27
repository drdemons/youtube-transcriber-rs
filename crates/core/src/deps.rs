use std::process::Command;

#[allow(dead_code)]
struct Tool {
    name: &'static str,
    check_args: &'static [&'static str],
    install_linux: &'static str,
    install_macos: &'static str,
}

const TOOLS: &[Tool] = &[
    Tool {
        name: "yt-dlp",
        check_args: &["--version"],
        install_linux: "pip install yt-dlp",
        install_macos: "brew install yt-dlp  (или: pip install yt-dlp)",
    },
    Tool {
        name: "ffmpeg",
        check_args: &["-version"],
        install_linux: "sudo apt install ffmpeg  (или: sudo dnf install ffmpeg)",
        install_macos: "brew install ffmpeg",
    },
];

/// Check that yt-dlp and ffmpeg are installed.
/// Returns an error with platform-specific install instructions if any are missing.
pub fn check_dependencies() -> Result<(), String> {
    let missing: Vec<&Tool> = TOOLS
        .iter()
        .filter(|tool| !is_installed(tool))
        .collect();

    if missing.is_empty() {
        return Ok(());
    }

    let mut msg = String::from("Missing required tools:\n\n");

    for tool in &missing {
        msg.push_str(&format!("  {} — not found\n", tool.name));

        #[cfg(target_os = "linux")]
        msg.push_str(&format!("    Install: {}\n\n", tool.install_linux));

        #[cfg(target_os = "macos")]
        msg.push_str(&format!("    Install: {}\n\n", tool.install_macos));

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        msg.push_str(&format!(
            "    Install yt-dlp: https://github.com/yt-dlp/yt-dlp\n\
             Install ffmpeg:  https://ffmpeg.org/download.html\n\n"
        ));
    }

    Err(msg)
}

fn is_installed(tool: &Tool) -> bool {
    Command::new(tool.name)
        .args(tool.check_args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}
