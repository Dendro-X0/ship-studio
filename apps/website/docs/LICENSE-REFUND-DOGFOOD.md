# License delivery + refund dogfood (W4)

End-to-end path without card data on Ship Studio servers. Polar holds payment;
you issue a **local key file** and email it (or attach as a Polar benefit).

Prereq: [POLAR-SETUP.md](./POLAR-SETUP.md) (product · checkout URL · redirects).

## A. Buy → license (dogfood)

1. Complete a **sandbox** Polar checkout → lands on `/checkout/success`.  
2. Copy Polar **order id** + buyer **email** from the Polar dashboard.  
3. Issue a key file:

```bash
# optional: stable keys across re-issue
# setx SHIP_LICENSE_SECRET "long-random-string"

python scripts/issue-license.py issue --email buyer@example.com --order polar_xxx
```

4. Email the `.license` file to the buyer (template below), **or** attach it as a
   Polar digital benefit / custom fulfillment note.  
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

1. Buyer opens `/account` → Polar customer portal (or emails `PUBLIC_SUPPORT_EMAIL`
   with order id) within `PUBLIC_REFUND_WINDOW_DAYS`.  
2. Maintainer approves the refund **in Polar**.  
3. Record local revoke (does not remote-wipe the machine):

```bash
python scripts/issue-license.py revoke --order polar_xxx --note "polar refund <id>"
```

4. Ledger line lands in `.ship-licenses/ledger.jsonl` (gitignored).  
5. Confirm with buyer: payment returned; stop using the software; local files remain theirs.

## C. Prove checklist

| Step | Expected |
|------|----------|
| `/pricing` → Polar checkout | Sandbox charge |
| `/checkout/success` | Download + license CTA |
| `issue-license.py issue` | Writes `.license` + ledger `issue` |
| Email / attach file | Buyer has key file |
| Portal refund | Polar shows refunded |
| `issue-license.py revoke` | Ledger `revoke` |
| `/legal/refunds` | Window + portal + email match env |

Live money requires maintainer Polar credentials (not stored in git).
