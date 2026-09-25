# Dogfood shelf

Offline and Advanced Publish walks. Scripts live under `scripts/`; these docs explain the operator path.

| Document | Role |
|----------|------|
| [DOGFOOD-guide.md](./DOGFOOD-guide.md) | General guide / paste path |
| [DOGFOOD-signet.md](./DOGFOOD-signet.md) | Signet-focused walk |
| [DOGFOOD-orbit-assess-api.md](./DOGFOOD-orbit-assess-api.md) | Orbit / assess-api style host |

Scripts: `scripts/dogfood-advanced-publish.sh|.ps1` · `scripts/dogfood-advanced-walk.sh` · `scripts/dogfood-offline.sh` · **`scripts/publish-fast.sh|.ps1`** (Continue chain) · **`scripts/harbor-reset.sh|.ps1`** (demo fixture wipe)

**Public demos:** bind [`fixtures/harbor`](../fixtures/harbor/) (Desktop + Docs). Advanced multipath dogfood stays on `fixtures/advanced-dogfood`.

### Fast path (minimal clicks)

```bash
# Burn Auto gates; stops at Human/Open — then Open → Confirm on that gate
bash scripts/publish-fast.sh fixtures/advanced-dogfood --mode general --intent local --chain 20
# Windows:
# powershell -ExecutionPolicy Bypass -File scripts/publish-fast.ps1 -Project fixtures/advanced-dogfood
# or:
shipctl publish --mode general --intent local --project . continue --chain 20
```

Desktop: bind folder → **Continue** (primary). Human gates show **Needs Open**.
