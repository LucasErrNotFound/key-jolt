# KeyJolt

KeyJolt adds custom sound feedback to keyboard presses and mouse clicks. Pick your own audio files, save them as presets, and keep the app running in the background while using other applications. Inspired by the *Welcome to the Game* franchise, it is built with GPUI-Kit.

## Features

- Create, edit, select, import, and delete keyboard and mouse presets.
- Choose full-size, TKL, or compact keyboard layouts and optionally synchronize selections between layouts.
- Assign sounds to keyboard keys or left, right, and middle mouse clicks.
- Preview files and choose sequential or random playback when multiple sounds are assigned.
- Control keyboard and mouse volume/mute independently, or disable all input-triggered sounds.
- Toggle the app globally with **Alt + Shift + M**.
- Choose light/dark appearance and bundled themes from the title-bar settings button.
- Keep sounds running with the window hidden; reopen or exit through the system tray.

Settings and presets are stored locally. The global listener keeps transient input state for sound triggers and the shortcut; application source does not record typed text or print input/event logs.

## Getting started

If you already have a Windows release build, open `key-jolt.exe`. Source-build instructions are below.

1. Choose **Create preset** in the home page's keyboard or mouse section.
2. Enter a name and add audio with **Upload Files** or drag files into the editor.
3. For a keyboard preset, choose a layout, select keys, and check the sounds to use. For a mouse preset, select a button and check its sounds; repeat for other buttons.
4. Use the play button beside a file to preview it. Choose a playback mode when at least two relevant sounds are selected or assigned.
5. Save. The home page selects the saved preset for input-triggered playback.
6. Leave **App Status** active and adjust each section's volume/mute as needed.

Use **Edit** for the selected keyboard preset or **Configure** for the selected mouse preset. The back button prompts before discarding unsaved changes.

### Controls and background behavior

| Control | Effect |
| --- | --- |
| **App Status** / **Alt + Shift + M** | Toggle keyboard and mouse sound triggers together. |
| Keyboard/mouse volume and mute | Control each sound group independently. |
| **Sequential** | Cycle through the applicable sound list. |
| **Random** | Choose from the applicable sound list. |
| **Shift + mouse wheel** over the keyboard canvas | Scroll the keyboard horizontally. |
| Window close button | Hide and keep running when the tray is available. If tray creation failed, closing exits. |
| Tray menu → **Open KeyJolt** | Restore the window. |
| Tray menu → **Exit** | Stop the app and background sounds. |

Holding a key does not continuously replay its sound from OS key repeat. Mouse wheel input and extra mouse buttons currently have no sound bindings. Preview controls operate separately from global sound triggers. Automatic startup is not configured by the application.

### Audio files

The build enables MP3, PCM WAV, FLAC, Ogg/Vorbis, and AAC/MP4 decoding through [rodio's codec features](https://docs.rs/crate/rodio/0.22.2/features). AAC in `.m4a` containers is covered by the MP4 decoder. The picker also accepts some extensions whose codecs are not enabled; acceptance does not guarantee playback. Preview before saving. Converting unsupported files to PCM WAV or MP3 is a practical fallback.

### Sharing and backups

Saved custom presets are ZIP packages containing `preset.json` and their sound assets. Copy a saved ZIP to share it; on another machine, choose **Import preset** in the matching keyboard or mouse section. Keep the archive intact. A ZIP of arbitrary audio files is not a KeyJolt preset.

Settings live in `settings.json`; packages live under `presets/keyboard/` and `presets/mouse/`. The default parent directories come from `ProjectDirs::from("com", "KeyJolt", "key-jolt")`:

| Platform | Default app-data directory |
| --- | --- |
| Windows | `%APPDATA%\KeyJolt\key-jolt\data` |
| macOS | `~/Library/Application Support/com.KeyJolt.key-jolt` |
| Linux | `~/.local/share/key-jolt`, or `$XDG_DATA_HOME/key-jolt` when configured. |

These follow the [directories crate's conventions](https://docs.rs/directories/6.0.0/directories/struct.ProjectDirs.html). Playback extracts sounds into the system temporary directory under `KeyJolt/preset-cache/`; back up saved ZIPs rather than the temporary cache. Exit KeyJolt before copying the app-data directory for a backup.

## Platform notes

| Platform | Requirements and limitations |
| --- | --- |
| Windows | Primary target. Standard release builds open without an extra console window. |
| macOS | Grant Accessibility permission to KeyJolt, or its launching terminal, in **System Settings → Privacy & Security → Accessibility**. |
| Linux | Global sounds require X11. The window may open under Wayland while global listening remains unavailable. Tray support depends on the desktop environment. |

An available audio output device is required

## Build from source

Install current stable Rust with [rustup](https://rust-lang.org/tools/install/). Keep the repository's `Cargo.lock` and `assets/themes/`: dependencies are locked and themes are embedded during compilation. Consult the bundled GPUI Kit installation section of `docs/gpui-kit/llms-full.txt` for version-specific native requirements.

### Windows

Install Visual Studio 2022 Build Tools or Community with **Desktop development with C++**, MSVC, and a Windows SDK. Make CMake available in the build terminal. Use the Rust MSVC toolchain.

```powershell
git clone https://github.com/LucasErrNotFound/key-jolt.git
cd key-jolt
cargo run --locked
```

Build and launch the release executable:

```powershell
cargo build --release --locked
.\target\release\key-jolt.exe
```

The Windows GUI subsystem is selected in `src/main.rs` when debug assertions are disabled, the standard release configuration. Debug builds retain the default console subsystem. `cargo run --release` still uses the existing terminal for Cargo's build output; launching the rebuilt release executable directly opens the app without creating a console. See the [Rust Reference](https://doc.rust-lang.org/reference/runtime.html#the-windows_subsystem-attribute).

### Linux

The following combines GPUI Kit's documented Ubuntu 24.04 dependencies with KeyJolt's X11 listener, audio, tray, and native-dialog requirements. Other distributions may use different package names:

```sh
sudo apt-get update
sudo apt-get install -y build-essential pkg-config cmake clang \
  libfontconfig-dev libwayland-dev libwebkit2gtk-4.1-dev \
  libxkbcommon-x11-dev libx11-xcb-dev libssl-dev libzstd-dev \
  libvulkan1 vulkan-validationlayers libx11-dev libxi-dev \
  libxtst-dev libasound2-dev libgtk-3-dev libxdo-dev \
  libappindicator3-dev
```

Run in a graphical X11 session with a working graphics driver, audio device, and desktop portal for native dialogs. Rendering uses Vulkan on Linux; its runtime package alone does not install a GPU driver.

```sh
git clone https://github.com/LucasErrNotFound/key-jolt.git
cd key-jolt
cargo run --locked
cargo build --release --locked
./target/release/key-jolt
```

### macOS

Follow GPUI Kit's documented requirement of macOS 15 or later and install Xcode Command Line Tools:

```sh
xcode-select --install
git clone https://github.com/LucasErrNotFound/key-jolt.git
cd key-jolt
cargo run --locked
```

Grant Accessibility permission as described above and restart if global sounds remain unavailable.

## Troubleshooting

| Problem | What to check |
| --- | --- |
| No sound on presses/clicks | App Status is active, a configured preset is selected, sounds are checked/assigned, and group/system volume is audible and unmuted. |
| Preview fails | Try a known-good PCM WAV/MP3 file and check the output device. Extensions alone do not identify supported codecs. |
| Preview works, global sounds do not | Check saved assignments, macOS Accessibility permission, and Linux session type. Use X11 on Linux. |
| The app disappeared after closing | Choose **Open KeyJolt** from its tray menu. Use **Exit** there to stop it. |
| Import fails | Choose a KeyJolt ZIP of the matching kind and read the in-app message. Malformed packages and unsafe archive paths are rejected. |
| A console still opens on Windows | Rebuild with `cargo build --release --locked` and launch `target/release/key-jolt.exe`. Older/debug executables can still use the console subsystem. |
| Theme JSON is missing during compilation | Restore the repository's `assets/themes/` files; they are compile-time inputs. |

## Development

The shell coordinates events and services; editors own their state, commands, dialogs, and rendering. Persistence and background services are independent of feature views.

| Location | Responsibility |
| --- | --- |
| `src/app/` | Startup, shell, title bar, and appearance controls. |
| `src/features/home/` | Preset selection and app/volume settings UI. |
| `src/features/keyboard/`, `src/features/mouse/` | Editors and workflows. |
| `src/features/preset_editor/` | Shared editor status and file-filter types. |
| `src/presets/` | Domain models, repository, ZIP storage, validation, and restoration. |
| `src/settings/`, `src/persistence.rs` | Settings, themes, data paths, and atomic writes. |
| `src/audio/`, `src/input/`, `src/runtime.rs` | Playback, global input, and coordination. |
| `src/platform/` | Tray and OS adapters. |
| `assets/`, `src/app_assets.rs` | Bundled resources and icons. |

Follow the official [Coding Guides](https://gpui-kit.com/docs/coding-guides/) and [Design Guides](https://gpui-kit.com/docs/design-guides/) before editing.

Run checks from the repository root:
```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
```

Tests cover input/selection logic, settings, and preset persistence. Native dialogs, tray behavior, audio devices, and visual interactions also need manual checks on each OS. Report bugs through [GitHub Issues](https://github.com/LucasErrNotFound/key-jolt/issues), including OS/session, reproduction steps, expected/actual behavior, and any in-app error.

## License :page_with_curl:
[<img src="https://www.gnu.org/graphics/gplv3-127x51.png" alt="GPLv3" >](http://www.gnu.org/licenses/gpl-3.0.html)

KeyJolt is licensed under the **GNU General Public License v3.0**. See the [LICENSE](https://github.com/LucasErrNotFound/key-jolt/blob/main/LICENSE)
file for details.

### What GPLv3 lets you do
- **Run** the software for any purpose
- **Study** the source code and **modify** it to suit your needs
- **Share** copies, and distribute your modified versions

### What GPLv3 requires when you distribute
- Provide the **complete corresponding source code**
- License derivative works under **GPLv3** so the same freedoms are preserved
- Include a **copy of the GPL license** with every distribution
- **State any changes** you made, with a date, in the modified files

This is a plain-language summary, not legal advice. The full license text in the
[LICENSE](https://github.com/LucasErrNotFound/key-jolt/blob/main/LICENSE) file is what governs.

For more information, visit https://www.gnu.org/licenses/gpl-3.0.html.