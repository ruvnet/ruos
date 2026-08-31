//! ruos-computeruse-mcp — a minimal Model Context Protocol (stdio, JSON-RPC 2.0)
//! server that gives an attached AI client full computer-use + CLI/console
//! control of the ruOS desktop:
//!   • GUI    — screenshot (scrot), mouse move/click/drag, type, key, scroll
//!   • console— run_shell: run an arbitrary command on the desktop, get output
//!   • system — system_action: named actions (install/optimize/update/status/
//!              restart) proxied to the desktop executor on 127.0.0.1:17870
//!
//! Transport is stdio: the client (Claude Code / Claude Desktop) launches this
//! over SSH, so the security boundary is *who can open the SSH session*, not an
//! open network port. Actions themselves are deliberately unrestricted (operator
//! choice, ADR-018). The process must run in the desktop's X session — the
//! launcher sets DISPLAY=:0 and XAUTHORITY; we default DISPLAY as a fallback.
//!
//! No async runtime, no MCP SDK: a fixed, small tool surface hand-wired against
//! the current spec (initialize / tools/list / tools/call / ping) so it compiles
//! clean to a static musl binary and tracks nothing it doesn't control.

use std::io::{BufRead, Write};
use std::process::Command;

use base64::Engine;
use serde_json::{json, Value};

const NAME: &str = "ruos-mcp";
const VERSION: &str = "0.1.0";
const EXECUTOR: &str = "http://127.0.0.1:17870/api/command";
const RESOLUTION: &str = "http://127.0.0.1:17870/api/desktop/resolution";
const SHOT: &str = "/tmp/ruos-cu-shot.png";

/// Run a shell command, returning (success, merged stdout+stderr).
fn sh(cmd: &str) -> (bool, String) {
    match Command::new("sh").arg("-lc").arg(cmd).output() {
        Ok(o) => {
            let mut s = String::from_utf8_lossy(&o.stdout).into_owned();
            let e = String::from_utf8_lossy(&o.stderr);
            if !e.trim().is_empty() {
                if !s.is_empty() && !s.ends_with('\n') {
                    s.push('\n');
                }
                s.push_str(&e);
            }
            (o.status.success(), s)
        }
        Err(e) => (false, format!("failed to spawn: {e}")),
    }
}

fn as_i(v: &Value, k: &str) -> Option<i64> {
    v.get(k).and_then(|x| x.as_i64())
}
fn as_s<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(|x| x.as_str()).unwrap_or("")
}

/// Escape a string for safe single-quoted embedding in a shell command.
fn shq(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn text(s: impl Into<String>) -> Value {
    json!({ "content": [{ "type": "text", "text": s.into() }] })
}
fn text_err(s: impl Into<String>) -> Value {
    json!({ "content": [{ "type": "text", "text": s.into() }], "isError": true })
}

/// The tool catalog advertised to `tools/list`.
fn tools() -> Value {
    let n = |name: &str, desc: &str, props: Value, req: Value| {
        json!({
            "name": name,
            "description": desc,
            "inputSchema": { "type": "object", "properties": props, "required": req }
        })
    };
    let xy = json!({
        "x": { "type": "integer", "description": "X pixel (0 = left)" },
        "y": { "type": "integer", "description": "Y pixel (0 = top)" }
    });
    json!([
        n("screenshot", "Capture the desktop and return it as a PNG image.", json!({}), json!([])),
        n("screen_size", "Return the screen resolution as 'WIDTH HEIGHT'.", json!({}), json!([])),
        n("cursor_position", "Return the current mouse position as 'x y'.", json!({}), json!([])),
        n("mouse_move", "Move the mouse to (x, y).", xy.clone(), json!(["x", "y"])),
        n("left_click", "Left-click. If x,y given, move there first.", xy.clone(), json!([])),
        n("right_click", "Right-click. If x,y given, move there first.", xy.clone(), json!([])),
        n("middle_click", "Middle-click. If x,y given, move there first.", xy.clone(), json!([])),
        n("double_click", "Double left-click. If x,y given, move there first.", xy.clone(), json!([])),
        n("left_click_drag", "Press left at (x,y), drag to (to_x,to_y), release.",
            json!({
                "x": {"type":"integer"}, "y": {"type":"integer"},
                "to_x": {"type":"integer"}, "to_y": {"type":"integer"}
            }),
            json!(["x","y","to_x","to_y"])),
        n("type_text", "Type a string of text at the current focus.",
            json!({ "text": {"type":"string"} }), json!(["text"])),
        n("key", "Press a key or chord in xdotool syntax, e.g. 'Return', 'ctrl+c', 'alt+Tab'.",
            json!({ "keys": {"type":"string"} }), json!(["keys"])),
        n("scroll", "Scroll the wheel. direction up|down|left|right, amount = clicks.",
            json!({
                "direction": {"type":"string","enum":["up","down","left","right"]},
                "amount": {"type":"integer","description":"wheel clicks (default 3)"}
            }),
            json!(["direction"])),
        n("wait", "Sleep for ms milliseconds (max 10000).",
            json!({ "ms": {"type":"integer"} }), json!(["ms"])),
        n("run_shell", "Run an arbitrary shell command on the desktop and return its output. Full CLI/console access.",
            json!({ "command": {"type":"string","description":"the shell command line"} }),
            json!(["command"])),
        n("system_action", "Run a named ruOS system action via the desktop executor. action = install|optimize|update|status|restart; args = e.g. a tool name ('codex') or 'storage'.",
            json!({
                "action": {"type":"string","description":"install | optimize | update | status | restart"},
                "args": {"type":"string","description":"e.g. 'codex', 'storage', 'ruview-server'"}
            }),
            json!(["action"])),
        n("desktop_resolution", "Change the desktop resolution AFTER launch (server-side, via xrandr on the ruOS virtual display) — not a client-window resize. Give either width+height, or a preset (720p|1080p|qxga|1440p). Optional refresh (default 60 Hz). Custom sizes are bounded (even, 640..3840 x 480..2160) and rejected if they exceed the dummy driver's ~300 MHz pixel-clock ceiling.",
            json!({
                "width": {"type":"integer","description":"pixels wide (with height); even, 640..3840"},
                "height": {"type":"integer","description":"pixels tall (with width); even, 480..2160"},
                "preset": {"type":"string","description":"720p | 1080p | qxga | 1440p (use instead of width/height)"},
                "refresh": {"type":"integer","description":"refresh Hz (default 60)"}
            }),
            json!([])),
    ])
}

/// Take a screenshot via whichever tool is present; return base64 PNG or an error.
fn capture() -> Result<String, String> {
    let _ = std::fs::remove_file(SHOT);
    let cmd = format!(
        "scrot -o -q 70 {s} 2>/dev/null || gnome-screenshot -f {s} 2>/dev/null || import -silent -window root {s} 2>/dev/null",
        s = SHOT
    );
    let (_ok, _out) = sh(&cmd);
    match std::fs::read(SHOT) {
        Ok(bytes) if !bytes.is_empty() => {
            Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
        }
        _ => Err("screenshot failed — no scrot/gnome-screenshot/imagemagick, or no X display".into()),
    }
}

/// Execute one tool call. Returns the MCP `result` object.
fn call_tool(name: &str, args: &Value) -> Value {
    // Move-first helper for the click tools.
    let move_prefix = |args: &Value| -> String {
        match (as_i(args, "x"), as_i(args, "y")) {
            (Some(x), Some(y)) => format!("xdotool mousemove {x} {y} && "),
            _ => String::new(),
        }
    };
    match name {
        "screenshot" => match capture() {
            Ok(b64) => json!({ "content": [{ "type": "image", "data": b64, "mimeType": "image/png" }] }),
            Err(e) => text_err(e),
        },
        "screen_size" => {
            let (ok, out) = sh("xdotool getdisplaygeometry");
            if ok { text(out.trim().to_string()) } else { text_err(out) }
        }
        "cursor_position" => {
            let (ok, out) = sh("xdotool getmouselocation --shell | sed -n '1,2p' | cut -d= -f2 | paste -sd' '");
            if ok { text(out.trim().to_string()) } else { text_err(out) }
        }
        "mouse_move" => {
            let (x, y) = (as_i(args, "x").unwrap_or(0), as_i(args, "y").unwrap_or(0));
            let (ok, out) = sh(&format!("xdotool mousemove {x} {y}"));
            if ok { text(format!("moved to {x} {y}")) } else { text_err(out) }
        }
        "left_click" | "right_click" | "middle_click" | "double_click" => {
            let btn = match name { "right_click" => 3, "middle_click" => 2, _ => 1 };
            let click = if name == "double_click" {
                "xdotool click --repeat 2 1".to_string()
            } else {
                format!("xdotool click {btn}")
            };
            let (ok, out) = sh(&format!("{}{click}", move_prefix(args)));
            if ok { text(format!("{name} done")) } else { text_err(out) }
        }
        "left_click_drag" => {
            let (x, y) = (as_i(args, "x").unwrap_or(0), as_i(args, "y").unwrap_or(0));
            let (tx, ty) = (as_i(args, "to_x").unwrap_or(0), as_i(args, "to_y").unwrap_or(0));
            let cmd = format!(
                "xdotool mousemove {x} {y} mousedown 1 mousemove {tx} {ty} mouseup 1"
            );
            let (ok, out) = sh(&cmd);
            if ok { text(format!("dragged {x},{y} -> {tx},{ty}")) } else { text_err(out) }
        }
        "type_text" => {
            let t = as_s(args, "text");
            let (ok, out) = sh(&format!("xdotool type --clearmodifiers -- {}", shq(t)));
            if ok { text("typed") } else { text_err(out) }
        }
        "key" => {
            let k = as_s(args, "keys");
            if k.is_empty() { return text_err("keys required"); }
            let (ok, out) = sh(&format!("xdotool key --clearmodifiers -- {}", shq(k)));
            if ok { text(format!("pressed {k}")) } else { text_err(out) }
        }
        "scroll" => {
            let dir = as_s(args, "direction");
            let amt = as_i(args, "amount").unwrap_or(3).clamp(1, 100);
            let btn = match dir { "up" => 4, "down" => 5, "left" => 6, "right" => 7, _ => 5 };
            let (ok, out) = sh(&format!("xdotool click --repeat {amt} {btn}"));
            if ok { text(format!("scrolled {dir} x{amt}")) } else { text_err(out) }
        }
        "wait" => {
            let ms = as_i(args, "ms").unwrap_or(0).clamp(0, 10_000) as u64;
            std::thread::sleep(std::time::Duration::from_millis(ms));
            text(format!("waited {ms}ms"))
        }
        "run_shell" => {
            let c = as_s(args, "command");
            if c.is_empty() { return text_err("command required"); }
            let (_ok, out) = sh(c);
            text(if out.trim().is_empty() { "(no output)".into() } else { out })
        }
        "system_action" => {
            let action = as_s(args, "action");
            let a = as_s(args, "args");
            if action.is_empty() { return text_err("action required"); }
            let body = json!({ "action": action, "args": a }).to_string();
            let cmd = format!(
                "curl -s --max-time 300 -H 'Content-Type: application/json' -d {} {}",
                shq(&body), EXECUTOR
            );
            let (_ok, out) = sh(&cmd);
            text(if out.trim().is_empty() {
                "no response from the desktop executor (is ruos-welcome running?)".into()
            } else {
                out
            })
        }
        "desktop_resolution" => {
            // Build the JSON body from whatever was supplied and POST it to the
            // executor's typed resolution endpoint (which validates + runs xrandr).
            let mut body = serde_json::Map::new();
            if let Some(w) = as_i(args, "width") {
                body.insert("width".into(), json!(w));
            }
            if let Some(h) = as_i(args, "height") {
                body.insert("height".into(), json!(h));
            }
            let p = as_s(args, "preset");
            if !p.is_empty() {
                body.insert("preset".into(), json!(p));
            }
            if let Some(r) = as_i(args, "refresh") {
                body.insert("refresh".into(), json!(r));
            }
            if body.is_empty() {
                return text_err("give width+height or a preset (720p|1080p|qxga|1440p)");
            }
            let payload = Value::Object(body).to_string();
            let cmd = format!(
                "curl -s --max-time 60 -H 'Content-Type: application/json' -d {} {}",
                shq(&payload),
                RESOLUTION
            );
            let (_ok, out) = sh(&cmd);
            text(if out.trim().is_empty() {
                "no response from the desktop executor (is ruos-welcome running?)".into()
            } else {
                out
            })
        }
        other => text_err(format!("unknown tool '{other}'")),
    }
}

fn reply(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}
fn reply_err(id: Value, code: i64, msg: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": msg } })
}

fn handle(msg: &Value) -> Option<Value> {
    let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = msg.get("id").cloned();
    // Notifications (no id) get no response.
    let Some(id) = id else { return None; };
    match method {
        "initialize" => {
            let pv = msg
                .get("params")
                .and_then(|p| p.get("protocolVersion"))
                .and_then(|v| v.as_str())
                .unwrap_or("2025-06-18")
                .to_string();
            Some(reply(id, json!({
                "protocolVersion": pv,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": NAME, "version": VERSION }
            })))
        }
        "ping" => Some(reply(id, json!({}))),
        "tools/list" => Some(reply(id, json!({ "tools": tools() }))),
        "tools/call" => {
            let params = msg.get("params").cloned().unwrap_or(json!({}));
            let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            if name.is_empty() {
                return Some(reply_err(id, -32602, "missing tool name"));
            }
            Some(reply(id, call_tool(name, &args)))
        }
        _ => Some(reply_err(id, -32601, "method not found")),
    }
}

fn main() {
    // Fallback so xdotool/scrot find the desktop X session even if the launcher
    // forgot to export it. The launcher normally sets both from the gdm session.
    if std::env::var("DISPLAY").is_err() {
        std::env::set_var("DISPLAY", ":0");
    }
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break; };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<Value>(line) else {
            // Malformed JSON — cannot know the id; skip per JSON-RPC.
            continue;
        };
        if let Some(resp) = handle(&msg) {
            if writeln!(stdout, "{}", resp).is_err() {
                break;
            }
            let _ = stdout.flush();
        }
    }
}
