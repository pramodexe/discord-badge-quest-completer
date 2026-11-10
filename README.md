# Discord Badge and Quest Completer

A desktop app for **Windows and Linux** that makes Discord show you as playing a game, without installing the game.

- **Finish Discord Quests and earn Orbs** without owning the game.
- **Climb the Game Variety badge.** The Game Variety page plays random Discord-detectable games one at a time (100 games, 1 minute each, 5 seconds apart by default) and fills a 100-cell punch card with each game's icon.
- **Raise your Game Time badge** by starting any game you like and leaving it running.

Both badges are shown with their ten levels, and there is a built-in guide in English and Sinhala.

> [!NOTE]
> Discord does not update badge progress straight away. New games and hours can take up to 24 to 48 hours to show on your profile. The app cannot read your badge level from Discord. The optional "Starting point" boxes let you type in the numbers from your profile.

## Download and install

Get the file for your system from the [Releases page](https://github.com/pramodexe/discord-badge-quest-completer/releases/latest). Each release lists SHA-256 checksums in `SHA256SUMS.txt`, so you can check your download.

| Your system | Download |
|---|---|
| Windows 10 / 11 | `…-windows-portable.exe` (no install) or `…-windows-setup.exe` (installer) |
| Debian, Ubuntu, Linux Mint, Pop!_OS, Zorin | `…_amd64.deb` |
| Arch, CachyOS, EndeavourOS, Manjaro, Garuda | `…-x86_64.AppImage` |
| Fedora, openSUSE | `…x86_64.rpm` or the AppImage |
| Any other 64-bit Linux | `…-x86_64.AppImage` |

### Windows

**Portable:** put `discord-badge-quest-completer-…-windows-portable.exe` in a folder you own (for example Downloads or Desktop, not `C:\Program Files`) and double-click it. It keeps its `games` folder next to itself.

**Installer:** run `discord-badge-quest-completer-…-windows-setup.exe` and follow the steps. It installs WebView2 for you if it is missing.

The app is not code-signed, so Windows may say "Windows protected your PC" or "unknown publisher". Choose **More info**, then **Run anyway**. The portable version needs WebView2, which comes with Windows 11 and up-to-date Windows 10. On an older system, [install WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2) or use the installer.

### Debian, Ubuntu, Linux Mint, Pop!_OS

Double-click the `.deb` file to install it with your software center, or open a terminal in your Downloads folder and run:

```bash
sudo apt install ./discord-badge-quest-completer_*_amd64.deb
```

Everything it needs is installed with it. Open **Discord Badge and Quest Completer** from your app menu.

### Arch, CachyOS, EndeavourOS, Manjaro (and any other Linux)

Use the AppImage. It is a single file with nothing to install:

```bash
sudo pacman -S --needed fuse2 gtk3            # once: lets AppImages run (often already installed)
chmod +x discord-badge-quest-completer-*-x86_64.AppImage
./discord-badge-quest-completer-*-x86_64.AppImage
```

You can also make it executable from your file manager (right-click, Properties, "Allow executing as program") and then double-click it.

On other systems, install FUSE 2 and GTK 3 the same way the first time: Ubuntu 22.04 and later `sudo apt install libfuse2 libgtk-3-0` (on Ubuntu 24.04 the package is `libfuse2t64`), Fedora `sudo dnf install fuse fuse-libs gtk3`.

### Fedora, openSUSE

```bash
sudo dnf install ./discord-badge-quest-completer-*.x86_64.rpm      # Fedora
sudo zypper install ./discord-badge-quest-completer-*.x86_64.rpm   # openSUSE
```

### The startup check

Each time the app opens it checks that everything is ready, and only says something when a step needs your attention:

- **Discord desktop app**: is Discord open, and is it a version that can see games?
- **Games folder**: can the app save its stand-in games?
- **Stand-in game program**: does it start on this system? (On Linux it needs GTK 3, and the check shows the exact install command for your system if it is missing.)

You can run it again any time with **Check setup** at the bottom of the app.

## Discord on Linux

Discord on Linux finds games by looking at the programs that are running. **Use the normal Discord desktop app**:

| How Discord is installed | Works? |
|---|---|
| Arch / CachyOS package (`sudo pacman -S discord`) | Yes |
| `.deb` or `.tar.gz` from [discord.com/download](https://discord.com/download) | Yes |
| Snap (`snap install discord`) | Reported to work |
| Flatpak (Flathub, `com.discordapp.Discord`) | May not work: its sandbox can hide other programs from it |
| Vesktop, WebCord, Legcord, or Discord in a web browser | Not supported: they do not detect games the same way |

If the app says you are playing but Discord does not show it, check which Discord you have first. The startup check tells you when it finds the Flatpak or a third-party app.

## Using it

1. Keep the Discord desktop app open and signed in on the same computer.
2. In Discord, open **Settings, then Activity Privacy**, and turn on sharing your detected activity.
3. **Library**: search for a game, add it, press Play. A small window with the game's name opens, and Discord shows you as playing it. Stop it, or close that window, when you are done.
4. **Game Variety**: set how many games, minutes per game and break, then press Start. One game runs at a time, and each is closed before the next starts. If a game cannot be closed, the run stops instead of starting another.

## Screenshots

The numbers in these screenshots are sample data, so you can see how progress looks.

**Game Variety**: the punch card fills with each game's icon. Picked games show dimmed until they are played.

![Game Variety page with the punch card](docs/screenshots/game-variety.png)

**Badge levels and the run list**: your progress to the next level, and the games picked for this run.

![Badge levels and the run list](docs/screenshots/badge-levels-and-run.png)

**Library**: search any detectable game, start or stop it by hand, and see the Game Time levels.

![Library page](docs/screenshots/library.png)

**Guide**: what the app is for and how to start, in English and Sinhala.

![Guide page](docs/screenshots/guide.png)

![The two badges and their levels](docs/screenshots/guide-badges.png)

## How it works

Discord works out what you are playing by looking for running programs. This app creates small stand-in programs named like real games and runs them. Each shows a small window with the game's icon. Discord sees the program and shows you as playing. Nothing is injected into Discord, and the app never asks for your Discord login or token.

The stand-ins only do this: stay open. They are kept in a `games` folder you can delete at any time:

- **Windows:** next to the app (portable) or in the install folder.
- **Linux:** `~/.local/share/discord-badge-quest-completer/games`.

The stand-in program is built into the app, so the app is a single file with nothing else to keep next to it.

The app talks to only a few places: Discord's public list of detectable games (and a GitHub mirror of it), and Discord's image server for game icons. There are no ads and no tracking.

## Troubleshooting

- **Linux: the app window is blank or white.** The app already turns on the usual fix (`WEBKIT_DISABLE_DMABUF_RENDERER=1`). If it still happens, start it from a terminal and share the output in an issue.
- **Linux: the AppImage does not start and mentions FUSE.** Install FUSE 2 (see the Arch / other Linux steps above).
- **Discord does not show the game.** Check Activity Privacy in Discord, check which Discord you have (see [Discord on Linux](#discord-on-linux)), and give Discord up to a minute to notice.
- **Windows: "Windows protected your PC".** Choose More info, then Run anyway. The app is not code-signed.

## Build from source

You need Node 20+, [pnpm](https://pnpm.io), Rust ([rustup.rs](https://rustup.rs)), and the build libraries for your system.

**Windows:** the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for Windows, including the C++ build tools.

**Arch / CachyOS / Manjaro:**

```bash
sudo pacman -S --needed base-devel rust nodejs pnpm webkit2gtk-4.1 gtk3 libayatana-appindicator librsvg
```

**Debian / Ubuntu (22.04 or newer):**

```bash
sudo apt install build-essential curl file libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev libxdo-dev
```

**Fedora:**

```bash
sudo dnf install webkit2gtk4.1-devel gtk3-devel openssl-devel libappindicator-gtk3-devel librsvg2-devel libxdo-devel
sudo dnf group install c-development
```

Then:

```bash
pnpm install
pnpm sync:runner        # builds the stand-in program for your system (needed before any app build)
pnpm tauri dev          # run in development (the first build takes several minutes)
pnpm tauri build        # build the app and its packages
```

`pnpm tauri build` on Linux makes the AppImage, `.deb` and `.rpm` in `src-tauri/target/release/bundle/`. On Windows it makes the installer there, and the portable app is `src-tauri/target/release/discord-quest-completer.exe`.

When you run `pnpm tauri dev`, use the app window that opens once the build is done. The `localhost:1420` page in a web browser is only a preview: it does not start any games.

Tests:

```bash
pnpm test               # TypeScript unit tests
cd src-tauri && cargo test
```

Project layout: `src/` is the Vue interface, `src-tauri/` is the Rust app, `src-win/` is the Windows stand-in program (C++ with no C runtime, so it stays small), and `src-linux/` is the Linux stand-in program (Rust and GTK 3).

### Making a release

The **Build and release** GitHub Actions workflow builds every download and creates a draft release with checksums. Run it from the Actions tab, or push a version tag:

```bash
git tag v1.1.0 && git push origin v1.1.0
```

Update `version` in `src-tauri/tauri.conf.json` first. The Linux files are built on Ubuntu 22.04 so they also run on newer systems.

## Credits and license

This is a fork of [draxctrl/discord-badge-quest-completer](https://github.com/draxctrl/discord-badge-quest-completer) with Linux support added. That project is an enhanced version of [Discord Quest Completer](https://github.com/markterence/discord-quest-completer) by Mark Terence Tiglao. If you find the original useful, consider supporting [the author](https://github.com/markterence).

Released under the [MIT License](LICENSE), which keeps the original copyright notice.

## Disclaimer

This tool is intended for educational purposes and personal use. Faking game activity may go against Discord's Terms of Service. Please respect Discord's terms of service, partners, game publishers and advertisers' rights when using this application.

The creators and maintainers of this project are not liable for any damages, account suspensions, or other consequences that may arise from using this software. Use at your own risk.

Discord is a registered trademark of Discord Inc. It is referenced on this open-source project for descriptive and definition purposes only and does not imply any affiliation, sponsorship, or endorsement by Discord Inc in any way.
