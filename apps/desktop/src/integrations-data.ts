/** Integration wizard catalog (Payments · Email). */

import type { IntegrationWizard } from "./types";

export const INTEGRATION_WIZARDS: IntegrationWizard[] = [
  {
    id: "polar",
    group: "Payments",
    title: "Polar",
    blurb: "Open products → copy checkout URL. Studio never creates SKUs.",
    provider: "polar",
    openUrl: "https://polar.sh/dashboard",
    openLabel: "Open Polar",
    docsUrl: "https://docs.polar.sh",
    needsPublic: true,
    openLinks: [
      { label: "Products", url: "https://polar.sh/dashboard" },
      { label: "Settings / tokens", url: "https://polar.sh/settings" },
    ],
    steps: [
      "Products → create a one-time product; set success → /checkout/success and cancel → /checkout/cancel.",
      "Copy checkout + customer portal URLs into the website host as PUBLIC_POLAR_CHECKOUT_URL and PUBLIC_POLAR_PORTAL_URL — Studio does not write them.",
      "Optional: Put POLAR_WEBHOOK_SECRET on the deploy host via Env Put (never paste into Studio).",
      "Continue publishing → Confirm listing.polar when that checkpoint is current. Refunds stay on Polar.",
    ],
  },
  {
    id: "stripe",
    group: "Payments",
    title: "Stripe",
    blurb: "Open Products / Payment Links. Studio never creates charges.",
    provider: "stripe",
    openUrl: "https://dashboard.stripe.com",
    openLabel: "Open Stripe",
    docsUrl: "https://docs.stripe.com",
    needsPublic: true,
    openLinks: [
      { label: "Products", url: "https://dashboard.stripe.com/products" },
      { label: "Payment Links", url: "https://dashboard.stripe.com/payment-links" },
      { label: "API keys", url: "https://dashboard.stripe.com/apikeys" },
    ],
    steps: [
      "Products / Payment Links → create or update the listing yourself.",
      "API keys → Put STRIPE_* names on the deploy host via Env — values never enter Studio.",
      "Continue publishing → Confirm listing.stripe when Publish shows it.",
    ],
  },
  {
    id: "gumroad",
    group: "Payments",
    title: "Gumroad",
    blurb: "Open products. Studio never creates listings.",
    provider: "gumroad",
    openUrl: "https://app.gumroad.com",
    openLabel: "Open Gumroad",
    docsUrl: "https://gumroad.com/help",
    needsPublic: true,
    openLinks: [
      { label: "Products", url: "https://app.gumroad.com/products" },
      { label: "Dashboard", url: "https://app.gumroad.com" },
    ],
    steps: [
      "Products → create or update the product yourself.",
      "Copy the product URL if the site needs it (paste outside Studio).",
      "Continue publishing → Confirm listing.gumroad when current.",
    ],
  },
  {
    id: "lemon",
    group: "Payments",
    title: "Lemon Squeezy",
    blurb: "Open store / products. Studio never sends fulfillment mail.",
    provider: "lemon",
    openUrl: "https://app.lemonsqueezy.com",
    openLabel: "Open Lemon",
    docsUrl: "https://docs.lemonsqueezy.com",
    needsPublic: true,
    openLinks: [
      { label: "Dashboard", url: "https://app.lemonsqueezy.com" },
      { label: "Products", url: "https://app.lemonsqueezy.com/products" },
    ],
    steps: [
      "Products → create/update product and checkout there.",
      "Confirm fulfillment email works in Lemon (Studio never sends mail).",
      "Continue publishing → Confirm listing.lemon when Publish shows it.",
    ],
  },
  {
    id: "paddle",
    group: "Payments",
    title: "Paddle",
    blurb: "More gates than Polar/Stripe — seed collapses catalog + token; Studio never holds pdl_ keys.",
    provider: "paddle",
    openUrl: "https://sandbox-vendors.paddle.com/",
    openLabel: "Open Paddle sandbox",
    docsUrl: "https://developer.paddle.com/build/checkout/build-overlay-checkout",
    needsPublic: true,
    openLinks: [
      { label: "Create sandbox account", url: "https://sandbox-vendors.paddle.com/signup" },
      { label: "Authentication (API key / test_…)", url: "https://sandbox-vendors.paddle.com/authentication" },
      { label: "Checkout configuration", url: "https://sandbox-vendors.paddle.com/checkout-settings" },
      { label: "Catalog (manual pri_…)", url: "https://sandbox-vendors.paddle.com/products" },
      { label: "Orders / refunds", url: "https://sandbox-vendors.paddle.com/orders" },
    ],
    steps: [
      "Sandbox is a separate login from paddle.com — Create sandbox account if needed.",
      "Preferred: Authentication → API key (pdl_sdbx_…, product/price/client_token write) → setx PADDLE_SANDBOX_API_KEY → python scripts/seed-paddle-solo-sandbox.py (writes PUBLIC_PADDLE_* outside Studio).",
      "Required: Checkout configuration → default payment link http://localhost:4321 — without it the overlay fails.",
      "Fallback (no seed): Catalog → pri_… · Authentication → Client-side tokens → paste PUBLIC_PADDLE_* into apps/website/.env yourself.",
      "pnpm website:dev → /pricing → test card 4242 4242 4242 4242 · CVC 100. Never put pdl_… in PUBLIC_*.",
      "Orders / refunds for paid tests. Continue publishing → Confirm listing.paddle when current.",
    ],
  },
  {
    id: "creem",
    group: "Payments",
    title: "Creem",
    blurb: "Open dashboard / API keys. Keys stay on the host.",
    provider: "creem",
    openUrl: "https://creem.io/dashboard",
    openLabel: "Open Creem",
    docsUrl: "https://docs.creem.io",
    needsPublic: true,
    openLinks: [
      { label: "Dashboard", url: "https://creem.io/dashboard" },
      { label: "API keys", url: "https://creem.io/dashboard/api-keys" },
    ],
    steps: [
      "Dashboard → create/update product and checkout yourself.",
      "API keys → Put CREEM_API_KEY / CREEM_WEBHOOK_SECRET on the deploy host via Env — not into Studio.",
      "Continue publishing → Confirm listing.creem when that checkpoint is current.",
    ],
  },
  {
    id: "waffo",
    group: "Payments",
    title: "Waffo",
    blurb: "Open Pancake merchant. Studio never creates products.",
    provider: "waffo",
    openUrl: "https://pancake.waffo.ai/merchant/auth/signin",
    openLabel: "Open Waffo",
    docsUrl: "https://docs.waffo.ai",
    needsPublic: true,
    openLinks: [
      { label: "Merchant sign-in", url: "https://pancake.waffo.ai/merchant/auth/signin" },
    ],
    steps: [
      "Merchant dashboard → create/update product and checkout yourself.",
      "Put WAFFO_MERCHANT_ID / WAFFO_PRIVATE_KEY on the deploy host via Env — not into Studio.",
      "Continue publishing → Confirm listing.waffo when that checkpoint is current.",
    ],
  },
  {
    id: "resend",
    group: "Email",
    title: "Resend",
    blurb: "Open API keys → Put on host. Closest to Deployment Put.",
    openUrl: "https://resend.com/api-keys",
    docsUrl: "https://resend.com/docs",
    openLabel: "Open API keys",
    needsPublic: false,
    openLinks: [
      { label: "API keys", url: "https://resend.com/api-keys" },
      { label: "Domains", url: "https://resend.com/domains" },
    ],
    steps: [
      "API keys → create a key (Studio never stores it).",
      "Put RESEND_API_KEY on the deploy host (Cloudflare / Vercel / Netlify) via Put key — paste only in the terminal.",
      "Send a test from the Resend UI. Studio never sends mail.",
      "Confirm gate when Publish shows env.sprint (Public) — or Continue publishing to that checkpoint.",
    ],
  },
];

/** Operator-facing “done” line — Verify stays human attest (S1.12). */
export const INTEGRATION_DONE_CRITERIA: Record<string, string> = {
  polar:
    "Done when: Polar checkout live · PUBLIC_POLAR_* set outside Studio · Confirm listing.polar (Verify = human attest).",
  stripe:
    "Done when: Stripe product/Payment Link live · keys on host outside Studio · Confirm listing.stripe (Verify = human attest).",
  gumroad:
    "Done when: Gumroad product/checkout live · Confirm listing.gumroad (Verify = human attest).",
  lemon:
    "Done when: Lemon checkout + fulfillment email work · Confirm listing.lemon (Verify = human attest).",
  paddle:
    "Done when: seed or manual PUBLIC_PADDLE_* · default payment link set · sandbox overlay works · Confirm listing.paddle (Verify = human attest).",
  creem:
    "Done when: Creem checkout live · CREEM_* on host outside Studio · Confirm listing.creem (Verify = human attest).",
  waffo:
    "Done when: Waffo checkout live · WAFFO_* on host outside Studio · Confirm listing.waffo (Verify = human attest).",
  resend:
    "Done when: RESEND_API_KEY on the deploy host · test send on Resend · Confirm env.sprint when current (Verify = human attest).",
};

/** Publish step to focus / Confirm from an Integrations wizard (S1.10 / S1.13). */
export const INTEGRATION_PUBLISH_STEP: Record<string, string> = {
  polar: "listing.polar",
  stripe: "listing.stripe",
  gumroad: "listing.gumroad",
  lemon: "listing.lemon",
  paddle: "listing.paddle",
  creem: "listing.creem",
  waffo: "listing.waffo",
  resend: "env.sprint",
};

/**
 * Host Put names (Cloudflare / Vercel / Netlify CLI) — never vendor-dashboard put.
 * Values paste only in the terminal. Website PUBLIC_* are still pasted outside Studio.
 */
export const INTEGRATION_HOST_PUT: Record<string, string> = {
  resend: "RESEND_API_KEY",
  stripe: "STRIPE_SECRET_KEY",
  creem: "CREEM_API_KEY",
  waffo: "WAFFO_PRIVATE_KEY",
  polar: "POLAR_WEBHOOK_SECRET",
  lemon: "LEMONSQUEEZY_API_KEY",
};
