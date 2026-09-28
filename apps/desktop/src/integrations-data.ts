/** Integration wizard catalog (Payments · Email). */

import type { IntegrationWizard } from "./types";

export const INTEGRATION_WIZARDS: IntegrationWizard[] = [
  {
    id: "polar",
    group: "Payments",
    title: "Polar",
    blurb: "Checkout, customer portal, and refunds.",
    provider: "polar",
    openUrl: "https://polar.sh/dashboard",
    needsPublic: true,
    steps: [
      "Public + Advanced intent — then Open dashboard (products).",
      "Create a one-time product; set success → /checkout/success and cancel → /checkout/cancel.",
      "Copy checkout + customer portal URLs into apps/website as PUBLIC_POLAR_CHECKOUT_URL and PUBLIC_POLAR_PORTAL_URL — Studio does not write them.",
      "Optional: Put POLAR_WEBHOOK_SECRET on the deploy host via Env Put (never paste the value into Studio).",
      "Continue publishing → Confirm listing.polar when that checkpoint is current. Refunds stay on Polar.",
    ],
  },
  {
    id: "stripe",
    group: "Payments",
    title: "Stripe",
    blurb: "Dashboard listing — no Payment Link creation from Studio.",
    provider: "stripe",
    openUrl: "https://dashboard.stripe.com",
    needsPublic: true,
    steps: [
      "Open Stripe and create or update the product / Payment Link yourself.",
      "Put publishable/secret names on the deploy host via Env — values never enter Studio.",
      "Continue publishing → Confirm the Stripe listing step when Publish shows it.",
    ],
  },
  {
    id: "gumroad",
    group: "Payments",
    title: "Gumroad",
    blurb: "Product listing on Gumroad.",
    provider: "gumroad",
    openUrl: "https://app.gumroad.com",
    needsPublic: true,
    steps: [
      "Open Gumroad and create or update the product.",
      "Copy the product URL if the site or listing needs it (paste outside Studio).",
      "Continue publishing → Confirm the Gumroad listing step when current.",
    ],
  },
  {
    id: "lemon",
    group: "Payments",
    title: "Lemon Squeezy",
    blurb: "Store and checkout on Lemon.",
    provider: "lemon",
    openUrl: "https://app.lemonsqueezy.com",
    needsPublic: true,
    steps: [
      "Open Lemon Squeezy — create/update product and checkout there.",
      "Confirm fulfillment email works in Lemon (Studio never sends mail).",
      "Continue publishing → Confirm the listing step when Publish shows it.",
    ],
  },
  {
    id: "paddle",
    group: "Payments",
    title: "Paddle",
    blurb: "Vendor dashboard for Paddle checkout.",
    provider: "paddle",
    openUrl: "https://vendors.paddle.com",
    needsPublic: true,
    steps: [
      "Open the Paddle vendor dashboard and create/update checkout yourself.",
      "Put any required keys on the deploy host via Env — not into Studio.",
      "Continue publishing → Confirm the listing step when current.",
    ],
  },
  {
    id: "resend",
    group: "Email",
    title: "Resend",
    blurb: "Transactional email API key.",
    openUrl: "https://resend.com/api-keys",
    openLabel: "Open API keys",
    needsPublic: false,
    steps: [
      "Open Resend → API keys and create a key (Studio never stores it).",
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
    "Done when: Paddle checkout live · keys on host outside Studio · Confirm listing.paddle (Verify = human attest).",
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
  resend: "env.sprint",
};
