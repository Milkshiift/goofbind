# goofbind

> A hard fork of [venbind](https://github.com/tuxinal/venbind) tailored for usage in [GoofCord](https://github.com/Milkshiift/GoofCord)

Goofbind is a cross-platform background service that monitors and manages global hotkeys. It communicates across stdin and stdout using JSON-formatted messages.

It can be intergrated with any language that supports spawning child processes and IO stream communication.

---

## Platforms

- **Windows**: Implements a low-level global hook (`WH_KEYBOARD_LL`) to capture keystrokes.
- **Linux (X11)**: Listens for raw input events via `rdev`.
- **Linux (Wayland)**: Integrates with the `org.freedesktop.portal.GlobalShortcuts` portal via the `ashpd` crate.

## Compiling
```sh
git clone https://github.com/Milkshiift/goofbind.git
cd goofbind

cargo build
```

On Linux, you will also need: `sudo apt-get install pkg-config libwayland-dev libxkbcommon-dev libx11-dev libxtst-dev` (Ubuntu/Debian)

## Usage

Start the executable from the command line, optionally providing an application identifier (for Wayland):

```bash
goofbind org.example.MyApp
```

### JSON API Protocol

#### 1. Configuring Keybinds (stdin)

To configure or overwrite active keybinds, send a JSON payload containing a `set_keybinds` command followed by a list of keybind objects.

```json
{
  "command": "set_keybinds",
  "keybinds": [
    {
      "id": "open-terminal",
      "name": "Open Terminal",
      "keycode": 84,
      "ctrl": true,
      "alt": true,
      "shift": false,
      "meta": false
    },
    {
      "id": "screenshot",
      "name": "Capture Screenshot",
      "keycode": 44,
      "ctrl": false,
      "alt": false,
      "shift": false,
      "meta": false
    }
  ]
}
```

For Windows and X11, standard Virtual Key codes are used (e.g., `84` represents `T`, `44` represents `Print Screen`). Under Wayland, keycode matching is handled directly by the portal manager during configuration; the specified identifiers are evaluated instead.

#### 2. Keybind Actions (stdout)

When a registered keybind is activated or deactivated, Goofbind writes a JSON line to standard output.

##### Pressed Event:
```json
{"event":"pressed","id":"open-terminal"}
```

##### Released Event:
```json
{"event":"released","id":"open-terminal"}
```

#### 3. Error Handling (stdout)

If a parsing error or a platform-specific issue occurs, an error payload is dispatched:

```json
{"event":"error","message":"Invalid JSON payload: ..."}
```