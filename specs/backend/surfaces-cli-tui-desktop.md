# Surfaces — Client · MCP · CLI

**Status:** Active — product law  
**Updated:** 2026-09-25  
**Owner:** `crates/shipctl` (CLI/MCP kernel) · `apps/desktop` (Client) · `shipctl tui` (terminal Client)  
**Canonical with:** [SCOPE-OF-SERVICE.md](../../docs/product/SCOPE-OF-SERVICE.md) · [PRODUCT.md](../../docs/product/PRODUCT.md)

## Goal

One shipping engine (`shipctl`), **three pathways**:

| Pathway | Surface | Audience |
|---------|---------|----------|
| **Client** | Desktop (Tauri); TUI | Humans — including founders without DevOps fluency |
| **MCP** | `shipctl mcp` | Agents (Cursor, etc.) assisting local release work |
| **CLI** | `shipctl` | Kernel + scripts + power users |

Same pattern as **CodaCtrl Studio + CodaCtrl MCP** or **Ghidra + Ghidra MCP**: UX for people, protocol for agents, shared local engine underneath.

## Invariants

1. Desktop and TUI are thin Clients — no second business logic; they invoke `shipctl`.  
2. MCP exposes the same contracts (`ship_publish`, `ship_guide`, `ship_portal`, …) — **no secret custody**; put/login remain human-initiated.  
3. CLI is the shared kernel. Excellence of the Client may make CLI invisible to buyers; it does not make CLI optional for the architecture.  
4. Portal semantics: navigate exact next human act (deep link / Put / Login CLI); human does OAuth/env. **Docs is secondary**, never the default setup path.  
5. Bridge does not call vendor HTTPS with secrets; shells may open URLs / spawn login or put CLIs on operator action.  
6. Vendor-onboarding theater (in-app coaches that still dump into encyclopedias) is **CANCELLED** — [vendor-handoff-coach-design](./vendor-handoff-coach-design.md).  
7. **Client honesty overhaul** (band #51) — Put/Login/deep-link CTA law — [ship-studio-overhaul-design](./ship-studio-overhaul-design.md) · [human-gate-catalog-design](./human-gate-catalog-design.md).

## Fee bar

| Pathway | May charge when |
|---------|-----------------|
| Client | Non-technical account-holder can finish human gates without jargon docs as the path |
| MCP | Agents usefully drive plan/continue/watch while humans keep keys |
| CLI alone | Not a consumer pitch — engineering/script surface |

## TUI

- Screens: Home · Providers · Portal · Ship wizard  
- Non-TTY: refuse with hint to use JSON CLI / MCP  

## Desktop (Client)

- Publish spine + Related panels + pulse CTA  
- Portal: Open / Docs / Login CLI; Env Put → interactive terminal  
- Must pass Client honesty (SCOPE success criterion 6)

## MCP

- Stdio tools for agents; operator retains control of keys and vendor accounts  
- Parallel to CodaCtrl MCP (enhance/optimize/debug) and Ghidra MCP (delegate RE) — **assist**, don’t replace human authority on irreversible acts  

## Proof

- L1: `cargo test -p shipctl`  
- L2: `shipctl portal` JSON · Desktop Portal from same JSON · MCP tool list  
- Product: SCOPE Delivery surfaces section matches this file  
