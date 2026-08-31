# ruos-mcp — the ruOS computer-use MCP server

A minimal [Model Context Protocol](https://modelcontextprotocol.io) server
(stdio, JSON-RPC 2.0) that gives an attached AI client full **computer-use +
CLI/console + system control** of a ruOS desktop:

- **GUI** — screenshot, mouse move/click/drag, type, key, scroll, wait (via `xdotool` / `scrot`).
- **Console** — `run_shell`: run an arbitrary command on the desktop and get its output.
- **System** — `system_action`: named ruOS actions (install / optimize / update / status / restart), proxied to the desktop executor.
- **Display** — `desktop_resolution`: change the desktop resolution after launch, server-side.

It is a single hand-wired Rust binary — **no async runtime, no MCP SDK** — that
compiles to a static musl binary and tracks nothing it doesn't control. The tool
surface is fixed and small, pinned to the current MCP spec
(`initialize` / `tools/list` / `tools/call` / `ping`).

Grounded in **ADR-018** (computer-use + system-control) and **ADR-022**
(desktop resolution API) of the `ruos-desktop` repo.

---

## Tools

| Tool | Category | What it does | Depends on |
|------|----------|--------------|------------|
| `screenshot` | GUI | Capture the desktop, return a PNG image | X session + `scrot`/`gnome-screenshot`/ImageMagick |
| `screen_size` | GUI | Return screen resolution as `WIDTH HEIGHT` | `xdotool` |
| `cursor_position` | GUI | Return mouse position as `x y` | `xdotool` |
| `mouse_move` | GUI | Move the mouse to `(x, y)` | `xdotool` |
| `left_click` / `right_click` / `middle_click` / `double_click` | GUI | Click (optionally move there first) | `xdotool` |
| `left_click_drag` | GUI | Press at `(x,y)`, drag to `(to_x,to_y)`, release | `xdotool` |
| `type_text` | GUI | Type a string at the current focus | `xdotool` |
| `key` | GUI | Press a key/chord in xdotool syntax (`Return`, `ctrl+c`, `alt+Tab`) | `xdotool` |
| `scroll` | GUI | Scroll `up`/`down`/`left`/`right` by N clicks | `xdotool` |
| `wait` | GUI | Sleep for `ms` milliseconds (max 10000) | — |
| `run_shell` | Console | Run an arbitrary shell command, return merged stdout+stderr | shell |
| `system_action` | System | Named ruOS action (`install`/`optimize`/`update`/`status`/`restart`) | **executor** on `127.0.0.1:17870` (ADR-018) |
| `desktop_resolution` | Display | Set desktop `WxH` after launch (width+height or preset `720p`/`1080p`/`qxga`/`1440p`) | **executor** with the resolution endpoint (ADR-022) |

**16 tools total.** Actions are deliberately **unrestricted** — there are no
per-action confirm dialogs. The security boundary is *who can open the transport*
(see below), not per-action gating. This is the operator-choice model from ADR-018.

### Prerequisites on the desktop

- An **X session** (`DISPLAY=:0`) with `xdotool` and a screenshot tool
  (`scrot`, `gnome-screenshot`, or ImageMagick's `import`). The launcher normally
  exports `DISPLAY` + `XAUTHORITY` from the desktop session; the binary falls back
  to `DISPLAY=:0` if unset.
- The **ruOS desktop executor** (`ruos-welcome-server`, ADR-016/018) listening on
  loopback `127.0.0.1:17870` — required only by `system_action` and
  `desktop_resolution`. The GUI + `run_shell` tools work without it.
  > `desktop_resolution` additionally requires the **ADR-022** resolution
  > endpoint on that executor. ADR-022 is *Proposed* (branch `feat/resolution-api`
  > of `ruos-desktop`) — the tool ships in the binary but no-ops with a clear
  > message if the endpoint isn't present.

---

## Build

```bash
cd mcp
cargo build --release
# → target/release/ruos-mcp
```

Optional static musl build (what the ruOS fleet deploys):

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

## Run / protocol

The server speaks JSON-RPC 2.0 over **stdio** — one JSON object per line in, one
per line out. Quick handshake:

```bash
printf '%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
  | ./target/release/ruos-mcp
```

Call a tool:

```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"run_shell","arguments":{"command":"uname -a"}}}
```

Supported methods: `initialize`, `ping`, `tools/list`, `tools/call`.
Notifications (no `id`) get no response, per JSON-RPC.

---

## Pointing an MCP client at a ruOS desktop

**Today — stdio over SSH.** The binary must run *inside the desktop's X session*,
so the client launches it over SSH. The security boundary is *who can open the SSH
session*, not an open network port — nothing here binds a socket.

Claude Code:

```bash
claude mcp add ruos -- ssh you@your-ruos-desktop \
  'DISPLAY=:0 XAUTHORITY=/run/user/1000/gdm/Xauthority /usr/local/bin/ruos-mcp'
```

Claude Desktop (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "ruos": {
      "command": "ssh",
      "args": [
        "you@your-ruos-desktop",
        "DISPLAY=:0 XAUTHORITY=/run/user/1000/gdm/Xauthority /usr/local/bin/ruos-mcp"
      ]
    }
  }
}
```

> Adjust `XAUTHORITY` to your desktop's session (the ruOS provisioning installs a
> launcher that sets both `DISPLAY` and `XAUTHORITY` from the gdm session; if you
> use it, the env prefix isn't needed).

**Roadmap — hosted MCP gateway.** A per-tenant MCP gateway on
`ruos.cognitum.one` (auth via Cognitum OAuth, id→desktop routing) would let a
client reach a ruOS desktop's MCP without an SSH account on the box. That gateway
does **not** exist yet — it depends on the OAuth tier (ADR-020, backlog). Until
then, stdio-over-SSH is the supported path.

---

## License

MIT — see [../LICENSE](../LICENSE).
