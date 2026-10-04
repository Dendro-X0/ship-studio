#!/usr/bin/env python3
"""Create sandbox Solo catalog + client token; merge into apps/website/.env.

Needs PADDLE_SANDBOX_API_KEY (pdl_sdbx_…). Never prints the API key.
Default payment link still must be set in the Paddle dashboard.
"""

from __future__ import annotations

import json
import os
import sys
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ENV_PATH = ROOT / "apps" / "website" / ".env"
BASE = "https://sandbox-api.paddle.com"
PRODUCT_NAME = "Ship Studio Solo"


def die(msg: str, code: int = 1) -> None:
    print(msg, file=sys.stderr)
    raise SystemExit(code)


def api_key() -> str:
    key = (os.environ.get("PADDLE_SANDBOX_API_KEY") or "").strip()
    if not key:
        die(
            "Set PADDLE_SANDBOX_API_KEY to a sandbox API key (pdl_sdbx_…). "
            "Create it at https://sandbox-vendors.paddle.com/authentication "
            "with product.write, price.write, and client_token.write. "
            "Do not put it in PUBLIC_* or git."
        )
    if not key.startswith("pdl_sdbx"):
        die("PADDLE_SANDBOX_API_KEY must be a sandbox key (pdl_sdbx_…), not live.")
    return key


def paddle(method: str, path: str, key: str, body: dict | None = None) -> dict:
    data = None if body is None else json.dumps(body).encode("utf-8")
    req = urllib.request.Request(
        BASE + path,
        data=data,
        method=method,
        headers={
            "Authorization": f"Bearer {key}",
            "Content-Type": "application/json",
            "Paddle-Version": "1",
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            return json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")
        die(f"Paddle {method} {path} failed HTTP {exc.code}: {detail[:800]}")


def list_all(key: str, path: str, extra: dict | None = None) -> list:
    out: list = []
    after = None
    while True:
        q = dict(extra or {})
        if after:
            q["after"] = after
        qs = urllib.parse.urlencode(q)
        payload = paddle("GET", f"{path}?{qs}" if qs else path, key)
        chunk = payload.get("data") or []
        out.extend(chunk)
        pag = payload.get("meta", {}).get("pagination") or {}
        if not pag.get("has_more"):
            break
        if not chunk:
            break
        after = chunk[-1].get("id")
        if not after:
            break
    return out


def upsert_catalog(key: str) -> tuple[str, str]:
    products = list_all(key, "/products", {"include": "prices"})
    existing = next((p for p in products if p.get("name") == PRODUCT_NAME), None)
    if existing:
        product_id = existing["id"]
        prices = existing.get("prices") or []
        one_time = next(
            (
                pr
                for pr in prices
                if not pr.get("billing_cycle")
                and (pr.get("unit_price") or {}).get("amount") == "2900"
            ),
            prices[0] if prices else None,
        )
        if one_time:
            return product_id, one_time["id"]
        created = paddle(
            "POST",
            "/prices",
            key,
            {
                "product_id": product_id,
                "description": "Ship Studio Solo one-time USD",
                "unit_price": {"amount": "2900", "currency_code": "USD"},
            },
        )
        return product_id, created["data"]["id"]

    product = paddle(
        "POST",
        "/products",
        key,
        {
            "name": PRODUCT_NAME,
            "tax_category": "digital-goods",
            "description": "Perpetual Solo license for the local shipping hub.",
        },
    )
    product_id = product["data"]["id"]
    price = paddle(
        "POST",
        "/prices",
        key,
        {
            "product_id": product_id,
            "description": "Ship Studio Solo one-time USD",
            "unit_price": {"amount": "2900", "currency_code": "USD"},
        },
    )
    return product_id, price["data"]["id"]


def upsert_client_token(key: str) -> str:
    tokens = list_all(key, "/client-tokens")
    named = next(
        (
            t
            for t in tokens
            if t.get("status") == "active" and t.get("name") == "Ship Studio website overlay"
        ),
        None,
    )
    if named and named.get("token"):
        return named["token"]
    created = paddle(
        "POST",
        "/client-tokens",
        key,
        {
            "name": "Ship Studio website overlay",
            "description": "Paddle.js overlay on /pricing (sandbox).",
        },
    )
    token = created["data"].get("token") or ""
    if not token:
        die("Paddle created a client token but did not return the secret; copy it from Authentication.")
    return token


def merge_env(updates: dict[str, str]) -> None:
    ENV_PATH.parent.mkdir(parents=True, exist_ok=True)
    lines = ENV_PATH.read_text(encoding="utf-8").splitlines() if ENV_PATH.exists() else []
    keys = set(updates)
    seen: set[str] = set()
    out: list[str] = []
    for line in lines:
        if "=" in line and not line.lstrip().startswith("#"):
            name = line.split("=", 1)[0].strip()
            if name in updates:
                out.append(f"{name}={updates[name]}")
                seen.add(name)
                continue
        out.append(line)
    for name, value in updates.items():
        if name not in seen:
            out.append(f"{name}={value}")
    ENV_PATH.write_text("\n".join(out) + "\n", encoding="utf-8")


def main() -> None:
    key = api_key()
    product_id, price_id = upsert_catalog(key)
    client_token = upsert_client_token(key)
    merge_env(
        {
            "PUBLIC_PADDLE_CLIENT_TOKEN": client_token,
            "PUBLIC_PADDLE_PRICE_ID": price_id,
            "PUBLIC_PADDLE_ENV": "sandbox",
            "PUBLIC_PADDLE_PRICE_LABEL": "$29",
        }
    )
    print(f"product {product_id}")
    print(f"price {price_id}")
    print(f"client_token {client_token[:5]}…")
    print(f"wrote {ENV_PATH.relative_to(ROOT)}")
    print("Still required once: default payment link http://localhost:4321")
    print("https://sandbox-vendors.paddle.com/checkout-settings")
    print("Then: pnpm website:dev → /pricing")


if __name__ == "__main__":
    main()
