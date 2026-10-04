# License delivery + refund dogfood (W4)

End-to-end path without card data on Ship Studio servers. Paddle holds payment;
you issue a **local key file** and email it.

Prereq: [PADDLE-SETUP.md](./PADDLE-SETUP.md) (product · client token · price id · overlay).

## A. Buy → license (dogfood)

1. Complete a **sandbox** Paddle overlay checkout → lands on `/checkout/success`.  
2. Copy Paddle **transaction id** + buyer **email** from the Paddle dashboard.  
3. Issue a key file:

```bash
# optional: stable keys across re-issue
# setx SHIP_LICENSE_SECRET "long-random-string"

python scripts/issue-license.py issue --email buyer@example.com --order paddle_xxx
```

4. Email the `.license` file to the buyer (template below).  
5. Buyer saves as `~/.ship/ship-studio.license` (see `/license` on the site).  
6. Buyer downloads the app from GitHub Releases and follows `/docs/start`.

Format: [docs/assets/license/FORMAT.md](../../../docs/assets/license/FORMAT.md).

> Desktop does not hard-gate on the file yet — W4 delivers entitlement + honesty.
> Runtime enforcement can land later without changing the file schema.

### Email template

```
Subject: Your Ship Studio Solo license

Thanks for purchasing Ship Studio.

Attached is your license key file (ship-studio.license).
Save it to:
  Windows: %USERPROFILE%\.ship\ship-studio.license
  macOS/Linux: ~/.ship/ship-studio.license

Download builds: https://github.com/Dendro-X0/ship-studio/releases
Docs: https://<your-host>/docs/start
Account / invoices: https://<your-host>/account

Refunds (14-day window): https://<your-host>/legal/refunds
```

## B. Refund path (dogfood)

1. Buyer emails `PUBLIC_SUPPORT_EMAIL` from `/legal/refunds` (or `/account`)
   with the Paddle transaction id within `PUBLIC_REFUND_WINDOW_DAYS`.  
2. Maintainer approves the refund **in Paddle → Transactions** (vendor dashboard).  
   Sandbox auto-approves in about 10 minutes; live usually needs Paddle approval.  
3. Record local revoke (does not remote-wipe the machine):

```bash
python scripts/issue-license.py revoke --order paddle_xxx --note "paddle refund <id>"
```

4. Ledger line lands in `.ship-licenses/ledger.jsonl` (gitignored).  
5. Confirm with buyer: payment returned; stop using the software; local files remain theirs.

## C. Prove checklist

| Step | Expected |
|------|----------|
| `/pricing` → Paddle overlay | Sandbox charge |
| `/checkout/success` | Download + license CTA |
| `issue-license.py issue` | Writes `.license` + ledger `issue` |
| Email / attach file | Buyer has key file |
| Portal refund | Paddle shows refunded |
| `issue-license.py revoke` | Ledger `revoke` |
| `/legal/refunds` | Window + portal + email match env |

Live money requires maintainer Paddle credentials (not stored in git).
