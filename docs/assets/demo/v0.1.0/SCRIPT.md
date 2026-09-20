# Feature demo — v0.1.0

Silent GIF shelf for `/demo`. Watermark: **local app · recorded**.

## Honesty

v0.1.0 GIFs are **stylized silent frames** of the Desktop Publish spine (bind → Open → Confirm → Next → Output Preview). They are not pixel-perfect screen captures. Replace with live Desktop recordings when ready; keep captions and file names stable.

Regenerate stylized set:

```bash
python scripts/generate-demo-gifs.py
```

## Presenter checklist (60–90s)

1. Open Desktop · bind this repo (or any ship-ready project).  
2. Publish: **Open** the first vendor door — browser, not Studio OAuth.  
3. Finish the vendor step · return · **Confirm**.  
4. **Next** through the spine · open **Output Preview** on a local artifact.  
5. Say the line: Studio sequences gates; you finish vendors.

## Captions (on-page)

| File | Title | Caption |
|------|-------|---------|
| `01-bind.gif` | Bind project | Pick a local folder. Studio stays on your machine. |
| `02-open.gif` | Publish · Open | Open the vendor door. Studio does not OAuth for you. |
| `03-confirm-next.gif` | Confirm → Next | You confirm each gate. Studio advances the spine. |
| `04-output-preview.gif` | Output Preview | Inspect local artifacts before the next Confirm. |

## Site paths

- Source of truth: `docs/assets/demo/v0.1.0/`  
- Served from: `/demo/v0.1.0/*.gif` (`apps/website/public/demo/v0.1.0/`)
