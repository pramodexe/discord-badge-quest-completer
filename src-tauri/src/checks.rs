//! The startup check: is everything in place for a fake game to show up in Discord?
//!
//! Things the app needs just to open its window (WebKitGTK and GTK on Linux,
//! WebView2 on Windows) cannot be checked here: without them the app does not
//! start. The .deb and .rpm packages pull those in, the AppImage carries its
//! own, and the Windows installer installs WebView2. This covers the rest.

use serde::Serialize;
use std::fs;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Warn,
    Fail,
}

#[derive(Serialize, Clone, Debug)]
pub struct Check {
    pub id: &'static str,
    pub title: String,
    pub status: Status,
    pub detail: String,
    /// What to do about it, if anything.
    pub fix: Option<String>,
    /// Commands to copy, labelled by system ("Arch", "Debian / Ubuntu"...).
    pub commands: Vec<(String, String)>,
}

impl Check {
    fn new(id: &'static str, title: &str, status: Status, detail: impl Into<String>) -> Self {
        Check { id, title: title.to_string(), status, detail: detail.into(), fix: None, commands: vec![] }
    }
    fn fix(mut self, fix: impl Into<String>) -> Self {
        self.fix = Some(fix.into());
        self
    }
    fn command(mut self, system: &str, command: &str) -> Self {
        self.commands.push((system.to_string(), command.to_string()));
        self
    }
}

#[tauri::command]
pub async fn system_check() -> Vec<Check> {
    tauri::async_runtime::spawn_blocking(run_checks)
        .await
        .unwrap_or_else(|e| vec![Check::new("internal", "Startup check", Status::Warn, format!("The check itself failed: {}", e))])
}

fn run_checks() -> Vec<Check> {
    let games = check_games_folder();
    // The stand-in is written into the games folder, so only test it if that works.
    let runner = match games.status {
        Status::Fail => None,
        _ => Some(check_runner()),
    };
    let mut checks = vec![check_discord(), games];
    checks.extend(runner);
    checks
}

// --- games folder ---------------------------------------------------------

fn check_games_folder() -> Check {
    let base = crate::games_base_dir();
    let title = "Games folder";
    let probe = base.join(".write-test");
    let result = fs::create_dir_all(&base).and_then(|_| fs::write(&probe, b"ok"));
    let _ = fs::remove_file(&probe);
    match result {
        Ok(()) => Check::new("games_folder", title, Status::Ok, format!("Fake games are saved in {}", base.display())),
        Err(e) => {
            let check = Check::new("games_folder", title, Status::Fail, format!("Cannot write to {}: {}", base.display(), e));
            if cfg!(target_os = "windows") {
                check.fix("Move the app to a folder you own, such as Downloads or Desktop (not C:\\Program Files), and open it again.")
            } else {
                check.fix("Make sure your home folder has free space and that you own this folder.")
            }
        }
    }
}

// --- stand-in program -----------------------------------------------------

fn check_runner() -> Check {
    let title = "Stand-in game program";
    let path = match crate::resolve_runner_template() {
        Ok(path) => path,
        Err(e) => return Check::new("runner", title, Status::Fail, e).fix("Free some disk space, then press Check again."),
    };

    #[cfg(target_os = "linux")]
    {
        return check_runner_starts(&path, title);
    }
    #[cfg(not(target_os = "linux"))]
    {
        Check::new("runner", title, Status::Ok, format!("Ready at {}", path.display()))
    }
}

/// Starts the stand-in with `--check`, exactly as a game would be started.
#[cfg(target_os = "linux")]
fn check_runner_starts(path: &std::path::Path, title: &str) -> Check {
    use std::time::{Duration, Instant};

    let mut cmd = std::process::Command::new(path);
    cmd.arg("--check").stdout(std::process::Stdio::null()).stderr(std::process::Stdio::piped());
    crate::clean_appimage_env(&mut cmd);

    let gtk_fix = |check: Check| {
        check
            .fix("The fake games need GTK 3. Install it, then press Check again.")
            .command("Arch / CachyOS / Manjaro", "sudo pacman -S --needed gtk3")
            .command("Debian / Ubuntu / Mint", "sudo apt install libgtk-3-0")
            .command("Fedora", "sudo dnf install gtk3")
    };

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => return gtk_fix(Check::new("runner", title, Status::Fail, format!("Could not start it: {}", e))),
    };
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => {
                return Check::new("runner", title, Status::Ok, "It starts correctly on this system.");
            }
            Ok(Some(_)) => {
                let mut err = String::new();
                if let Some(mut stderr) = child.stderr.take() {
                    use std::io::Read;
                    let _ = stderr.read_to_string(&mut err);
                }
                let err = err.trim();
                let detail = if err.is_empty() {
                    "It could not start.".to_string()
                } else {
                    format!("It could not start: {}", err.lines().last().unwrap_or(err))
                };
                return gtk_fix(Check::new("runner", title, Status::Fail, detail));
            }
            Ok(None) if started.elapsed() > Duration::from_secs(10) => {
                let _ = child.kill();
                let _ = child.wait();
                return Check::new("runner", title, Status::Warn, "It did not answer within 10 seconds.")
                    .fix("Press Check again. If it keeps happening, try a game from the Library anyway.");
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => return Check::new("runner", title, Status::Warn, format!("Could not tell whether it started: {}", e)),
        }
    }
}

// --- Discord --------------------------------------------------------------

#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(dead_code)] // not every kind exists on every OS
enum DiscordKind {
    /// The normal desktop app (.deb, .tar.gz, Arch package, Windows/macOS app).
    Native,
    Snap,
    Flatpak,
    /// Vesktop, WebCord, Legcord and similar: they do not detect games the same way.
    ThirdParty,
}

const DISCORD_TITLE: &str = "Discord desktop app";

fn check_discord() -> Check {
    let running = running_discords();
    let pick = |kind| running.iter().any(|k| *k == kind);

    if pick(DiscordKind::Native) {
        return Check::new("discord", DISCORD_TITLE, Status::Ok, "Discord is open.");
    }
    if pick(DiscordKind::Snap) {
        return Check::new("discord", DISCORD_TITLE, Status::Ok, "Discord (Snap) is open. If games do not show up, try the normal version from discord.com.");
    }
    if pick(DiscordKind::Flatpak) {
        return Check::new(
            "discord",
            DISCORD_TITLE,
            Status::Warn,
            "Discord is open, but it is the Flatpak version. Its sandbox can stop it from seeing other programs, so the fake games may not show up.",
        )
        .fix("If games do not show as playing, install the normal Discord instead and close the Flatpak one.")
        .command("Arch / CachyOS / Manjaro", "sudo pacman -S discord")
        .command("Debian / Ubuntu / Mint", "Download the .deb from discord.com/download, then: sudo apt install ./discord-*.deb")
        .command("Fedora", "Download the .tar.gz from discord.com/download, or use the RPM Fusion package: sudo dnf install discord");
    }
    if pick(DiscordKind::ThirdParty) {
        return Check::new(
            "discord",
            DISCORD_TITLE,
            Status::Warn,
            "A third-party Discord app (such as Vesktop or WebCord) is open. These may not detect games.",
        )
        .fix("Use the official Discord desktop app from discord.com for game detection.");
    }

    let check = Check::new(
        "discord",
        DISCORD_TITLE,
        Status::Warn,
        "Discord is not open. Discord in a web browser cannot see games, so the desktop app is needed.",
    )
    .fix("Open the Discord desktop app, sign in, then press Check again.");

    if flatpak_discord_installed() {
        check.fix("Open Discord, sign in, then press Check again. Note: you have the Flatpak version, which may not see the fake games; the normal version from discord.com works best.")
    } else {
        check
    }
}

#[cfg(target_os = "linux")]
fn running_discords() -> Vec<DiscordKind> {
    let mut found = vec![];
    let Ok(entries) = fs::read_dir("/proc") else { return found };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(pid) = name.to_str().filter(|n| n.bytes().all(|b| b.is_ascii_digit())) else { continue };
        let comm = fs::read_to_string(format!("/proc/{}/comm", pid)).unwrap_or_default();
        let exe = fs::read_link(format!("/proc/{}/exe", pid))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        if let Some(kind) = classify_linux_process(comm.trim(), &exe) {
            if !found.contains(&kind) {
                found.push(kind);
            }
        }
    }
    found
}

/// Tells a Discord process apart from the rest by its name and program path.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn classify_linux_process(comm: &str, exe: &str) -> Option<DiscordKind> {
    let lower = comm.to_ascii_lowercase();
    if ["vesktop", "webcord", "legcord", "armcord", "dorion", "goofcord"].contains(&lower.as_str()) {
        return Some(DiscordKind::ThirdParty);
    }
    if !["discord", "discordptb", "discordcanary", "discorddevelop"].contains(&lower.as_str()) {
        return None;
    }
    // A Flatpak app sees its own files under /app; a Snap lives under /snap.
    if exe.starts_with("/app/") {
        Some(DiscordKind::Flatpak)
    } else if exe.starts_with("/snap/") {
        Some(DiscordKind::Snap)
    } else {
        Some(DiscordKind::Native)
    }
}

#[cfg(target_os = "windows")]
fn running_discords() -> Vec<DiscordKind> {
    let output = crate::quiet_command("tasklist").args(["/FO", "CSV", "/NH"]).output();
    let text = output.map(|o| String::from_utf8_lossy(&o.stdout).to_ascii_lowercase()).unwrap_or_default();
    let native = ["\"discord.exe\"", "\"discordptb.exe\"", "\"discordcanary.exe\""].iter().any(|n| text.contains(n));
    let third = ["\"vesktop.exe\"", "\"legcord.exe\"", "\"webcord.exe\""].iter().any(|n| text.contains(n));
    let mut found = vec![];
    if native {
        found.push(DiscordKind::Native);
    }
    if third {
        found.push(DiscordKind::ThirdParty);
    }
    found
}

#[cfg(target_os = "macos")]
fn running_discords() -> Vec<DiscordKind> {
    let open = ["Discord", "Discord PTB", "Discord Canary"].iter().any(|name| {
        std::process::Command::new("pgrep").arg("-x").arg(name).output().map(|o| o.status.success()).unwrap_or(false)
    });
    if open { vec![DiscordKind::Native] } else { vec![] }
}

fn flatpak_discord_installed() -> bool {
    if !cfg!(target_os = "linux") {
        return false;
    }
    let mut places = vec![std::path::PathBuf::from("/var/lib/flatpak/app/com.discordapp.Discord")];
    if let Some(home) = std::env::var_os("HOME") {
        places.push(std::path::PathBuf::from(home).join(".local/share/flatpak/app/com.discordapp.Discord"));
    }
    places.iter().any(|p| p.exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tells_discord_installs_apart() {
        assert_eq!(classify_linux_process("Discord", "/opt/discord/Discord"), Some(DiscordKind::Native));
        assert_eq!(classify_linux_process("Discord", "/usr/share/discord/Discord"), Some(DiscordKind::Native));
        assert_eq!(classify_linux_process("DiscordCanary", "/opt/discord-canary/DiscordCanary"), Some(DiscordKind::Native));
        assert_eq!(classify_linux_process("Discord", "/app/discord/Discord"), Some(DiscordKind::Flatpak));
        assert_eq!(classify_linux_process("Discord", "/snap/discord/200/usr/share/discord/Discord"), Some(DiscordKind::Snap));
        assert_eq!(classify_linux_process("vesktop", "/opt/Vesktop/vesktop"), Some(DiscordKind::ThirdParty));
        assert_eq!(classify_linux_process("firefox", "/usr/lib/firefox/firefox"), None);
        assert_eq!(classify_linux_process("discord-quest-c", "/x/discord-quest-completer"), None);
    }
}
