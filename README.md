# ruOS

**An agentic cloud desktop.** ruOS is your own Linux computer in the cloud, with
the full [ruvnet](https://github.com/ruvnet) AI stack built in — reachable from
any browser, an installable iPad app, or a native macOS app. Nothing to install
to use it; a real desktop to drive when you want one.

This repo is the public control-and-extension surface for a ruOS desktop:

- **[`mcp/`](./mcp/)** — the ruOS **computer-use MCP server**: screenshot,
  keyboard/mouse, `run_shell`, `system_action`, and `desktop_resolution`. Point
  Claude (or any MCP client) at a ruOS desktop and let it drive.
- **[`skills/`](./skills/)** — the ruvnet-stack **skills** you run on a ruOS
  desktop: swarm orchestration, vector memory, self-learning hooks, and the
  RuVector memory substrate.

> Served at **[ruos.cognitum.one](https://ruos.cognitum.one)** · Powered by
> [Cognitum](https://cognitum.one).

---

## What ruOS is

A ruOS desktop is a full GNOME Linux desktop running on a cloud VM, streamed to
you over a remote-desktop transport. You reach it three ways, all against the
same desktop:

- **Browser** — a rebranded [noVNC](https://novnc.com) viewer at
  `ruos.cognitum.one/vnc.html`. No install, any device.
- **iPad** — an installable PWA (`ruos.cognitum.one/ipad/`); a native SwiftUI app
  is on the roadmap.
- **macOS** — the **ruOS Connect** app (native RustDesk-based viewer).

On top of the desktop sit two control surfaces this repo documents:

- The **MCP control plane** ([`mcp/`](./mcp/)) — an AI client drives the desktop
  (pixels, keyboard, shell, system actions) over stdio-over-SSH.
- The **skills** ([`skills/`](./skills/)) — the ruvnet stack (Ruflo + RuVector),
  invoked from a terminal or an attached MCP client.

## Architecture

```
   Browser (noVNC)  ┐
   iPad PWA         ├─►  ruos.cognitum.one  ──►  ruOS desktop VM
   macOS Connect    ┘     (viewer front)          (GNOME on Xorg)
                                                    │
   Claude / MCP client ── stdio over SSH ──►  ruos-mcp  ──► xdotool / scrot / shell
                                                    │
                                             executor (127.0.0.1:17870)
                                                    │  system_action, desktop_resolution
                                             ruvnet stack (Ruflo, RuVector) ── skills
```

- **Viewer front** — a viewer served over HTTPS (Caddy + Let's Encrypt). Works
  standalone (direct) today; a per-tenant OAuth + id→desktop routing tier is the
  hosted-mode direction (see Roadmap).
- **MCP control plane** — [`mcp/ruos-mcp`](./mcp/) exposes 16 computer-use tools.
  Transport is **stdio over SSH**: the security boundary is who can open the SSH
  session, not an open port.
- **Executor** — a desktop-local server on loopback `127.0.0.1:17870` runs named
  system actions and the resolution change; the MCP server proxies to it.
- **Skills** — the ruvnet stack preinstalled on the desktop; see
  [`skills/`](./skills/).

## OS

| Aspect | Detail |
|--------|--------|
| **Base** | Ubuntu 24.04 LTS (`ubuntu-2404-lts-amd64`) on a GCP Compute Engine VM |
| **Desktop** | GNOME on Xorg, with `xf86-video-dummy` (virtual display `DUMMY0` — no physical GPU/framebuffer on a GCE instance) |
| **Transport** | RustDesk (native/macOS) and x11vnc→noVNC (browser); server-side resolution via `xrandr` |
| **Stack** | ruvnet AI stack — Ruflo (`@claude-flow/cli`) + RuVector (`ruvector`), plus the `ruos-mcp` computer-use server |
| **Region** | GCP `northamerica-northeast2` (Toronto) — chosen for low interactive-streaming latency |

### Tiers (cpu / gpu)

One machine-profile variable selects the tier (ADR-009 of `ruos-desktop`):

| Tier | Machine | Encode | Notes |
|------|---------|--------|-------|
| **cpu** (default) | `e2-standard-4` (4 vCPU / 16 GB) | Software VP9 | Cost-sensitive; BBR/fq + tuned buffers, animations off |
| **gpu** (opt-in) | `g2-standard-4` (+ NVIDIA L4) | Hardware NVENC H.264/H.265 | ~6× the cpu cost; GPU also free for local inference |

A scheduled stop/start (weekday 07:00–23:00, Toronto) keeps the cpu tier around
~$50/mo (ADR-002). Flipping tiers is an in-place machine-type change, not a rebuild.

### Multi-tenant

The **direction** is a multi-tenant hosted SaaS: sign up, get your own private
ruOS desktop, reach it from any device. That model is **not fully shipped** —
today a desktop runs in single-operator mode; per-tenant OAuth (Cognitum OIDC),
id→desktop routing, signup capture, and usage metering are on the roadmap. The
viewer is designed so hosted auth is a **wrapping tier that can be enabled or
disabled**, never baked into the core viewer.

## Quickstart

```bash
# 1. Build the MCP server (see mcp/README.md for musl + deploy notes)
cd mcp && cargo build --release      # → target/release/ruos-mcp

# 2. Point Claude Code at a ruOS desktop (stdio over SSH)
claude mcp add ruos -- ssh you@your-ruos-desktop \
  'DISPLAY=:0 XAUTHORITY=/run/user/1000/gdm/Xauthority /usr/local/bin/ruos-mcp'

# 3. Wire the ruvnet skills into the session
claude mcp add claude-flow -- npx -y @claude-flow/cli@latest
```

Then ask Claude to screenshot the desktop, run a shell command, or spin up a
Ruflo swarm. See [`mcp/README.md`](./mcp/README.md) and
[`skills/README.md`](./skills/README.md) for the full surface.

## Real vs roadmap

**Real, today**

- The `ruos-mcp` server (16 tools) — builds, handshakes, and drives a desktop
  over stdio-over-SSH.
- The browser noVNC viewer, iPad PWA, and macOS Connect app against a live
  desktop.
- The ruvnet skills (`@claude-flow/cli`, `ruvector`) via `npx`.
- cpu / gpu machine tiers.

**Roadmap**

- `desktop_resolution` needs the ADR-022 executor endpoint (branch
  `feat/resolution-api`; ADR is *Proposed*).
- Multi-tenant hosted mode: Cognitum OAuth, id→desktop routing, signup backend,
  usage metering.
- A hosted MCP gateway (reach a desktop's MCP without an SSH account).
- A native iPad (SwiftUI) app and an in-desktop skills gallery.

## Repository layout

```
ruos/
├── mcp/            # ruos-mcp — the computer-use MCP server (Rust)
│   ├── src/main.rs
│   ├── Cargo.toml
│   └── README.md   # tool list, JSON-RPC/stdio usage, client setup
├── skills/         # ruvnet-stack skills as SKILL.md descriptors
│   ├── ruflo-swarm/   ruflo-memory/   ruflo-hooks/   ruvector/
│   └── README.md
├── LICENSE         # MIT
└── README.md
```

## License

MIT — see [LICENSE](./LICENSE).
