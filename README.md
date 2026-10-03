# KeyJolt

KeyJolt plays sounds for configured keyboard keys and mouse buttons while it runs in the background.

## Linux build dependencies

The global listener uses X11. Install the X11 and audio development packages before building on Ubuntu or Debian:

```sh
sudo apt-get install build-essential pkg-config libx11-dev libxi-dev libxtst-dev libasound2-dev libgtk-3-dev libxdo-dev libappindicator3-dev
```

Global input listening does not work in Wayland sessions. Use an X11 session for global sounds.

[WIP]

## License :page_with_curl:
[<img src="https://www.gnu.org/graphics/gplv3-127x51.png" alt="GPLv3" >](http://www.gnu.org/licenses/gpl-3.0.html)

KeyJolt is licensed under the **GNU General Public License v3.0**. See the [LICENSE](https://github.com/LucasErrNotFound/key-jolt/main/LICENSE)
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
[LICENSE](https://github.com/LucasErrNotFound/key-jolt/main/LICENSE) file is what governs.

For more information, visit https://www.gnu.org/licenses/gpl-3.0.html.