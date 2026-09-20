# Ship Studio license file — format v1

Local, file-based entitlement. Studio does **not** phone home to validate.
Cloudflare Worker / Polar webhook automation is optional later — W4 ships maintainer
issuance + buyer delivery (email attachment or download).

## File name

`ship-studio.license` (JSON)

Suggested paths (buyer picks one):

- `%USERPROFILE%\.ship\ship-studio.license` (Windows)
- `~/.ship/ship-studio.license` (macOS / Linux)
- Project `.ship/ship-studio.license` (optional per-repo copy)

## Schema

```json
{
  "schema": "ship-studio.license/v1",
  "product": "solo",
  "email": "buyer@example.com",
  "order_id": "polar_order_or_checkout_id",
  "issued_at": "2026-09-20T12:00:00Z",
  "key": "SS1-ABCD-EFGH-IJKL",
  "status": "active"
}
```

| Field | Rules |
|-------|--------|
| `schema` | Always `ship-studio.license/v1` |
| `product` | `solo` (only SKU in W4) |
| `email` | Polar checkout email |
| `order_id` | Polar order / checkout id |
| `issued_at` | ISO-8601 UTC |
| `key` | `SS1-` + three 4-char groups (A–Z0–9) |
| `status` | `active` \| `revoked` |

## Issue / revoke

```bash
python scripts/issue-license.py issue --email you@example.com --order polar_xxx
python scripts/issue-license.py revoke --order polar_xxx
```

Ledger (gitignored): `.ship-licenses/ledger.jsonl`

Set `SHIP_LICENSE_SECRET` for stable keys across re-issue; without it, keys are random (fine for sandbox dogfood).
