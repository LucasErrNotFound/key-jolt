# KeyJolt

KeyJolt plays sounds for configured keyboard keys and mouse buttons while it runs in the background.

## Linux build dependencies

The global listener uses X11. Install the X11 and audio development packages before building on Ubuntu or Debian:

```sh
sudo apt-get install build-essential pkg-config libx11-dev libxi-dev libxtst-dev libasound2-dev libgtk-3-dev libxdo-dev libappindicator3-dev
```

Global input listening does not work in Wayland sessions. Use an X11 session for global sounds.

[WIP]