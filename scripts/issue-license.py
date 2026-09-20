#!/usr/bin/env python3
"""Issue or revoke Ship Studio Solo license key files (W4).

Ledger: .ship-licenses/ledger.jsonl (gitignored).
Stable keys when SHIP_LICENSE_SECRET is set; otherwise random (sandbox OK).
"""

from __future__ import annotations

import argparse
import hashlib
import hmac
import json
import os
import re
import secrets
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LEDGER_DIR = ROOT / ".ship-licenses"
LEDGER = LEDGER_DIR / "ledger.jsonl"
SCHEMA = "ship-studio.license/v1"
PRODUCT = "solo"
ALPHABET = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789"  # no I/O/0/1


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def group4(raw: str) -> str:
    chars = re.sub(r"[^A-Z0-9]", "", raw.upper())
    chars = "".join(c if c in ALPHABET else ALPHABET[ord(c) % len(ALPHABET)] for c in chars)
    while len(chars) < 12:
        chars += secrets.choice(ALPHABET)
    return f"SS1-{chars[0:4]}-{chars[4:8]}-{chars[8:12]}"


def mint_key(email: str, order_id: str) -> str:
    secret = (os.environ.get("SHIP_LICENSE_SECRET") or "").strip()
    if secret:
        digest = hmac.new(
            secret.encode("utf-8"),
            f"{order_id}|{email}|{PRODUCT}".encode("utf-8"),
            hashlib.sha256,
        ).hexdigest()
        return group4(digest.upper())
    return group4(secrets.token_hex(8).upper())


def append_ledger(row: dict) -> None:
    LEDGER_DIR.mkdir(parents=True, exist_ok=True)
    with LEDGER.open("a", encoding="utf-8") as f:
        f.write(json.dumps(row, separators=(",", ":")) + "\n")


def cmd_issue(args: argparse.Namespace) -> int:
    email = args.email.strip().lower()
    order_id = args.order.strip()
    if not email or not order_id:
        print("email and order are required", file=sys.stderr)
        return 2

    license_doc = {
        "schema": SCHEMA,
        "product": PRODUCT,
        "email": email,
        "order_id": order_id,
        "issued_at": utc_now(),
        "key": mint_key(email, order_id),
        "status": "active",
    }

    out = Path(args.out) if args.out else Path(f"ship-studio-{order_id}.license")
    out.write_text(json.dumps(license_doc, indent=2) + "\n", encoding="utf-8")
    append_ledger({"event": "issue", "at": utc_now(), **license_doc, "path": str(out.resolve())})

    print(f"wrote {out.resolve()}")
    print(f"key {license_doc['key']}")
    if not (os.environ.get("SHIP_LICENSE_SECRET") or "").strip():
        print("note: SHIP_LICENSE_SECRET unset — key is random (sandbox OK)", file=sys.stderr)
    print("email buyer the file, or attach via Polar benefit. See apps/website/docs/LICENSE-REFUND-DOGFOOD.md")
    return 0


def cmd_revoke(args: argparse.Namespace) -> int:
    order_id = args.order.strip()
    if not order_id:
        print("order is required", file=sys.stderr)
        return 2
    row = {
        "event": "revoke",
        "at": utc_now(),
        "schema": SCHEMA,
        "product": PRODUCT,
        "order_id": order_id,
        "status": "revoked",
        "note": args.note or "refunded on Polar",
    }
    append_ledger(row)
    print(f"revoked order {order_id} in {LEDGER}")
    print("Polar refund is separate — complete it in the Polar dashboard.")
    return 0


def main() -> int:
    p = argparse.ArgumentParser(description="Ship Studio license issue/revoke (W4)")
    sub = p.add_subparsers(dest="cmd", required=True)

    issue = sub.add_parser("issue", help="Mint a ship-studio.license JSON file")
    issue.add_argument("--email", required=True)
    issue.add_argument("--order", required=True, help="Polar order / checkout id")
    issue.add_argument("--out", default="", help="Output path (default ship-studio-<order>.license)")
    issue.set_defaults(func=cmd_issue)

    revoke = sub.add_parser("revoke", help="Record revoke after Polar refund")
    revoke.add_argument("--order", required=True)
    revoke.add_argument("--note", default="")
    revoke.set_defaults(func=cmd_revoke)

    args = p.parse_args()
    return int(args.func(args))


if __name__ == "__main__":
    raise SystemExit(main())
