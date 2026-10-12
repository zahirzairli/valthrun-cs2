![Valthrun CS2 Logo](./img/logo.svg)
<p align="right">
<a href="https://discord.gg/ecKbpAPW5T">
<img src="https://discordapp.com/api/guilds/1135362291311849693/widget.png?style=shield">
</a>
</p>

Valthrun is an open source external Counter-Strike 2 read only kernel-level gameplay enhancer.  
That's a lot of descriptive words, but what does each of them mean?  
- `Valthrun` The name of this project
- `open source` This application is open source and for everyone to learn from
- `external` We do not inject any DLLs into the target process
- `read only` We do not write to the CS2 process in any way, therefore being impossible to detect by scanning the process memory
- `kernel` We do not use any user level WinAPIs in order to get information from the CS2 process
  
This project is mainly a fun example for exploring the Windows Kernel with [Rust](https://www.rust-lang.org) and exploring the world of game enhancements :)

# WARNING
Valthrun is **not** a plug 'n play solution.  
Instead Valthrun aims to achieve maximum stealth and stay invisible for other applications.  
Please read [How to use](https://wiki.valth.run/getting-started/) carefully and try troubleshooting issues on your own.  

  
# Features
Due to Valthrun being read-only (as of now), there are limitations on what features are possible to implement (eg. skin changer).
Regardless of this limitation, Valthrun supports the following features:  

- External radar  
  Publish your game to a radar server you host yourself (`radar-server-standalone`, default `ws://127.0.0.1:7229/publish`), or to the public server https://radar.valth.run with `--publish-url wss://radar.valth.run/publish`
- Player ESP  
  ESP with a lot of configuration possibilities like: `Skeleton`, `Boxes3D` and `Boxes2D` 
  - Configurable colors to distinguish between enemy and team players
  - ESP includes player health, a health bar, weapon and a lot more
- Bomb Info
  - Time until the bomb detonation
  - Defuser info such as a defuse timer
  - Bomb site where the bomb is located
- Trigger Bot  
  Shoot your enemy (or team mates) as soon they enter your crosshair
- Spectator info
  - List of player currently watching you / the observer target
- Stream proof by default  
  Never show your Valthrun overlay on any screen shares

To access Valthruns settings overlay press `PAUSE`.

## Planned Features
- Aimbot
- Player competitive ranks / wins

# How to use / Getting started
This fork needs no separate driver. Memory access is built into `controller.exe` (see `driver/`):
it opens CS2 with the standard Windows APIs (`OpenProcess`, `ReadProcessMemory`) and never writes to the game.

1. Install [Rust](https://rustup.rs/) and the [Visual Studio 2022 Build Tools](https://visualstudio.microsoft.com/downloads/)
   with the **Desktop development with C++** workload.
2. Download this repository (Code > Download ZIP) and extract it.
3. Open **x64 Native Tools Command Prompt for VS 2022**, go to the extracted folder and run:
   ```
   cargo build --release -p controller
   ```
4. Start CS2 with the launch option `-insecure` (VAC disabled).
5. Run `targetelease\controller.exe`. Press `PAUSE` to open the settings.

# Troubleshooting
- `Could not find CS2 process`: start CS2 before the controller.
- `Could not open the CS2 process for reading`: CS2 runs as administrator, so run the controller as administrator too.
- `LNK1104: cannot open file 'msvcrt.lib'`: build from the x64 Native Tools Command Prompt for VS 2022.

# VAC
The controller holds an open handle to CS2, draws an overlay on top of the game and can send simulated input.
VAC can see all of this, so only use it with `-insecure` (offline or against bots). Using it on VAC secured servers can get your account banned.

# Screenshots
![](./img/showcase_01.png)
![](./img/showcase_02.png)

# Help
You can find help on the official Valthrun Discord server:  
[![Discord Shield](https://discordapp.com/api/guilds/1135362291311849693/widget.png?style=shield)](https://discord.gg/ecKbpAPW5T)
