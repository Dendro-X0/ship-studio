#!/usr/bin/env python3
"""R4 Harbor demo capture via WebView2 CDP screenshots → 720×420 GIFs."""

from __future__ import annotations

import asyncio
import base64
import json
import shutil
import subprocess
import urllib.request
from pathlib import Path

import websockets

ROOT = Path(r"E:\Web Projects\ship-studio")
HARBOR = str(ROOT / "fixtures" / "harbor")
RAW = ROOT / "docs" / "assets" / "demo" / "v0.2.5" / "_raw"
OUT = ROOT / "docs" / "assets" / "demo" / "v0.2.5"
PUB = ROOT / "apps" / "website" / "public" / "demo" / "v0.2.5"
CDP_HTTP = "http://127.0.0.1:9223"


def list_ws() -> str:
    with urllib.request.urlopen(f"{CDP_HTTP}/json/list", timeout=3) as r:
        tabs = json.load(r)
    return next(t["webSocketDebuggerUrl"] for t in tabs if t.get("type") == "page")


class CDP:
    def __init__(self, ws):
        self.ws = ws
        self.id = 0
        self.pending: dict[int, asyncio.Future] = {}
        self._task: asyncio.Task | None = None

    async def start(self):
        self._task = asyncio.create_task(self._reader())
        await self.call("Page.enable")
        await self.call("Runtime.enable")

    async def close(self):
        if self._task:
            self._task.cancel()
            try:
                await self._task
            except asyncio.CancelledError:
                pass

    async def _reader(self):
        try:
            async for raw in self.ws:
                data = json.loads(raw)
                mid = data.get("id")
                if mid in self.pending:
                    fut = self.pending.pop(mid)
                    if "error" in data:
                        fut.set_exception(RuntimeError(data["error"]))
                    else:
                        fut.set_result(data.get("result"))
        except asyncio.CancelledError:
            raise
        except Exception as e:
            print("reader", e, flush=True)

    async def call(self, method, params=None, timeout=20.0):
        self.id += 1
        mid = self.id
        fut = asyncio.get_running_loop().create_future()
        self.pending[mid] = fut
        payload = {"id": mid, "method": method}
        if params is not None:
            payload["params"] = params
        await self.ws.send(json.dumps(payload))
        return await asyncio.wait_for(fut, timeout=timeout)

    async def call_noreply(self, method, params=None):
        """Send and do not wait (for reload / navigation)."""
        self.id += 1
        payload = {"id": self.id, "method": method}
        if params is not None:
            payload["params"] = params
        await self.ws.send(json.dumps(payload))

    async def eval(self, expr):
        r = await self.call(
            "Runtime.evaluate",
            {"expression": expr, "returnByValue": True},
        )
        if r and r.get("exceptionDetails"):
            raise RuntimeError(str(r["exceptionDetails"])[:500])
        return (r or {}).get("result", {}).get("value")

    async def screenshot(self) -> bytes:
        r = await self.call(
            "Page.captureScreenshot",
            {"format": "jpeg", "quality": 80, "fromSurface": True},
        )
        return base64.b64decode(r["data"])

    async def click(self, selector: str):
        box = await self.eval(
            f"""(() => {{
              const el = document.querySelector({selector!r});
              if (!el) return null;
              el.scrollIntoView({{block:'center'}});
              const r = el.getBoundingClientRect();
              return {{x:r.x+r.width/2, y:r.y+r.height/2}};
            }})()"""
        )
        if not box:
            raise RuntimeError(f"missing {selector}")
        await self._click_xy(box["x"], box["y"])

    async def click_text(self, text: str):
        box = await self.eval(
            f"""(() => {{
              const walk = [...document.querySelectorAll(
                'button,a,[role=button],.nav-item,.host-card,[data-host],.card,label,.chip'
              )];
              const el = walk.find(e => (e.textContent||'').replace(/\\s+/g,' ').includes({text!r}));
              if (!el) return null;
              el.scrollIntoView({{block:'center'}});
              const r = el.getBoundingClientRect();
              return {{x:r.x+r.width/2, y:r.y+r.height/2, t:(el.textContent||'').trim().slice(0,80)}};
            }})()"""
        )
        if not box:
            raise RuntimeError(f"text not found: {text}")
        print("  click", repr(box.get("t")), flush=True)
        await self._click_xy(box["x"], box["y"])

    async def _click_xy(self, x, y):
        for typ in ("mouseMoved", "mousePressed", "mouseReleased"):
            p = {"type": typ, "x": x, "y": y, "button": "left"}
            if typ != "mouseMoved":
                p["clickCount"] = 1
            await self.call("Input.dispatchMouseEvent", p)

    async def capture_period(self, seconds: float, fps: float = 5.0) -> list[bytes]:
        frames: list[bytes] = []
        n = max(3, int(seconds * fps))
        for _ in range(n):
            t0 = asyncio.get_running_loop().time()
            frames.append(await self.screenshot())
            dt = asyncio.get_running_loop().time() - t0
            await asyncio.sleep(max(0.0, (1.0 / fps) - dt))
        return frames


def frames_to_gif(name: str, frames: list[bytes]):
    if len(frames) < 3:
        raise RuntimeError(f"{name}: only {len(frames)} frames")
    shot_dir = RAW / f"frames_{name}"
    if shot_dir.exists():
        shutil.rmtree(shot_dir)
    shot_dir.mkdir(parents=True)
    for i, blob in enumerate(frames):
        (shot_dir / f"f{i:04d}.jpg").write_bytes(blob)
    mp4 = RAW / f"{name}.mp4"
    gif = OUT / f"{name}.gif"
    err = subprocess.run(
        [
            "ffmpeg",
            "-y",
            "-framerate",
            "8",
            "-i",
            str(shot_dir / "f%04d.jpg"),
            "-vf",
            "scale=720:420:flags=lanczos,fps=8,split[s0][s1];"
            "[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer:bayer_scale=3",
            "-loop",
            "0",
            str(gif),
        ],
        capture_output=True,
    )
    if err.returncode != 0:
        print(err.stderr[-500:].decode(errors="replace"), flush=True)
        raise RuntimeError(f"ffmpeg failed for {name}")
    # keep mp4 sidecar optional
    subprocess.run(
        [
            "ffmpeg",
            "-y",
            "-framerate",
            "8",
            "-i",
            str(shot_dir / "f%04d.jpg"),
            "-vf",
            "scale=720:420:flags=lanczos",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            str(mp4),
        ],
        capture_output=True,
    )
    shutil.copy2(gif, PUB / gif.name)
    print(f"OK {name}.gif {gif.stat().st_size/1e6:.2f}MB n={len(frames)}", flush=True)


async def connect() -> tuple:
    ws = await websockets.connect(list_ws(), max_size=40_000_000)
    c = CDP(ws)
    await c.start()
    return ws, c


async def seed_prefs(c: CDP, *, clear: bool, seed_recent: bool):
    if clear:
        await c.eval(
            """(() => {
              localStorage.removeItem('orbit-yard.last-project');
              localStorage.removeItem('orbit-yard.recent-projects');
              localStorage.setItem('orbit-yard.mode', 'advanced');
              localStorage.setItem('orbit-yard.intent', 'local');
              localStorage.setItem('orbit-yard.output-dock', 'hidden');
              localStorage.setItem('orbit-yard.offline', '0');
              localStorage.setItem('orbit-yard.include-deploy', '1');
              return 'cleared';
            })()"""
        )
    else:
        # Seed recents so a chip exists; auto-bind is flaky — we click the chip after reload
        await c.eval(
            f"""(() => {{
              const p = {HARBOR!r};
              localStorage.setItem('orbit-yard.recent-projects', JSON.stringify([p]));
              if ({str(seed_recent).lower()}) {{
                localStorage.setItem('orbit-yard.last-project', p);
              }} else {{
                localStorage.removeItem('orbit-yard.last-project');
              }}
              localStorage.setItem('orbit-yard.mode', 'advanced');
              localStorage.setItem('orbit-yard.intent', 'local');
              localStorage.setItem('orbit-yard.output-dock', 'hidden');
              localStorage.setItem('orbit-yard.offline', '0');
              localStorage.setItem('orbit-yard.include-deploy', '1');
              return 'seeded';
            }})()"""
        )


async def reload_page(clear: bool, seed_recent: bool = True):
    ws, c = await connect()
    try:
        await seed_prefs(c, clear=clear, seed_recent=seed_recent)
        await c.call_noreply("Page.reload", {"ignoreCache": True})
    finally:
        await c.close()
        await ws.close()
    await asyncio.sleep(4.0)


async def click_harbor_recent(c: CDP):
    """Bind Harbor via #next-ship-open dataset.path (calls bindProject; skips flaky prune/recents)."""
    clicked = await c.eval(
        f"""(() => {{
          const p = {HARBOR!r};
          const btn = document.querySelector('#next-ship-open');
          if (!btn) return {{ok:false, why:'no #next-ship-open'}};
          btn.dataset.path = p;
          btn.disabled = false;
          btn.click();
          return {{ok:true, text:(btn.textContent||'').trim().slice(0,40)}};
        }})()"""
    )
    print("bind click", clicked, flush=True)
    if not clicked or not clicked.get("ok"):
        raise RuntimeError("Harbor bind control missing")
    await asyncio.sleep(4.0)
    body = await c.eval("document.body.innerText.slice(0,500)")
    ok = bool(
        body
        and "harbor" in body.lower()
        and "no project bound" not in body.lower()
    )
    print(
        "bound?",
        ok,
        (body or "")[:200].encode("ascii", "replace").decode(),
        flush=True,
    )
    if not ok:
        raise RuntimeError("Harbor bind failed — aborting R4 capture")


async def shoot_clip(c: CDP, name: str, action, before=0.8, after=5.0):
    frames: list[bytes] = []
    frames += await c.capture_period(before)
    await action()
    frames += await c.capture_period(after)
    frames_to_gif(name, frames)


async def main():
    RAW.mkdir(parents=True, exist_ok=True)
    OUT.mkdir(parents=True, exist_ok=True)
    PUB.mkdir(parents=True, exist_ok=True)

    ws, c = await connect()
    try:
        one = await c.screenshot()
        (RAW / "_probe.jpg").write_bytes(one)
        print("probe", len(one), flush=True)
    finally:
        await c.close()
        await ws.close()

    # T1: empty Dashboard (no native Open dialog — it races WebView/recents), then recent→harbor
    print("T1 clear…", flush=True)
    await reload_page(clear=True)
    ws, c = await connect()
    try:
        a = await c.capture_period(2.0)
        print("t1a", len(a), flush=True)
    finally:
        await c.close()
        await ws.close()

    print("T1 seed + click harbor…", flush=True)
    await reload_page(clear=False, seed_recent=False)
    ws, c = await connect()
    try:
        # Re-seed after boot in case prune raced; then force chip render via history if needed
        mid = await c.capture_period(0.6)
        await click_harbor_recent(c)
        # Dismiss history/switcher overlays if any
        await c.eval(
            """(() => {
              const dlg = document.querySelector('#recent-history-dialog');
              if (dlg) dlg.hidden = true;
              const sw = document.querySelector('#project-switcher');
              if (sw) sw.hidden = true;
              return true;
            })()"""
        )
        b = await c.capture_period(4.0)
        frames_to_gif("01-bind", a + mid + b)

        async def t2():
            await c.click_text("Deployment")
            await asyncio.sleep(0.6)
            for label in ("Cloudflare", "Login CLI", "Open dashboard"):
                try:
                    await c.click_text(label)
                    break
                except Exception:
                    continue

        print("T2…", flush=True)
        await shoot_clip(c, "02-open", t2, before=0.6, after=4.5)

        async def t3():
            await c.click_text("Publish")
            await asyncio.sleep(0.6)
            for label in ("Continue", "Confirm", "Check", "Refresh"):
                try:
                    await c.click_text(label)
                    await asyncio.sleep(0.4)
                except Exception:
                    continue

        print("T3…", flush=True)
        await shoot_clip(c, "03-confirm-next", t3, before=0.6, after=5.0)

        async def t4():
            for label in ("Preview", "Output"):
                try:
                    await c.click_text(label)
                    break
                except Exception:
                    continue

        print("T4…", flush=True)
        await shoot_clip(c, "04-output-preview", t4, before=0.5, after=3.5)

        async def t5():
            await c.click_text("Sign")
            await asyncio.sleep(1.0)
            try:
                await c.click("#btn-back-view")
            except Exception:
                try:
                    await c.click_text("Publish")
                except Exception:
                    await c.click_text("Dashboard")

        print("T5…", flush=True)
        await shoot_clip(c, "05-midflight", t5, before=0.5, after=4.0)

        async def t6():
            await c.click_text("Sign")
            await asyncio.sleep(0.8)
            # Prefer official Open on a sign card — avoid "Open folder"
            opened = await c.eval(
                """(() => {
                  const btns = [...document.querySelectorAll('button')];
                  const el = btns.find(b => {
                    const t = (b.textContent||'').replace(/\\s+/g,' ').trim();
                    return t === 'Open' || t.startsWith('Open dashboard') || t === 'Open portal';
                  });
                  if (!el) return false;
                  el.click();
                  return true;
                })()"""
            )
            print("  sign open", opened, flush=True)
            await asyncio.sleep(1.2)
            try:
                await c.click("#btn-back-view")
            except Exception:
                pass

        print("T6…", flush=True)
        await shoot_clip(c, "06-sign-open", t6, before=0.5, after=5.0)

        await c.eval(
            """(() => {
              localStorage.setItem('orbit-yard.intent', 'public');
              const b = document.querySelector('#intent-public');
              if (b) b.click();
              return 'public';
            })()"""
        )
        await asyncio.sleep(0.4)

        async def t7():
            await c.click_text("Deployment")
            await asyncio.sleep(0.5)
            try:
                await c.click_text("Cloudflare")
            except Exception:
                pass
            await asyncio.sleep(0.4)
            # Click the Deploy action in the host panel (not nav)
            deployed = await c.eval(
                """(() => {
                  const btns = [...document.querySelectorAll('button')];
                  const el = btns.find(b => (b.textContent||'').trim() === 'Deploy' && !b.disabled);
                  if (!el) return false;
                  el.click();
                  return true;
                })()"""
            )
            print("  deploy", deployed, flush=True)
            await asyncio.sleep(7.0)
            for label in ("Open dashboard", "Open live"):
                try:
                    await c.click_text(label)
                    break
                except Exception:
                    continue

        print("T7…", flush=True)
        await shoot_clip(c, "07-deploy-open", t7, before=0.5, after=2.5)
    finally:
        await c.close()
        await ws.close()

    print("GIFS", sorted(p.name for p in OUT.glob("*.gif")), flush=True)


if __name__ == "__main__":
    asyncio.run(main())
