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
      "Optional: put POLAR_WEBHOOK_SECRET on the deploy host via Env / secrets put (never paste the value into Studio).",
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
    needsPublic: false,
    steps: [
      "Open Resend → API keys and create a key.",
      "Env / tokens → Put RESEND_API_KEY on the deploy host (or wrangler secret) — do not paste the value into Studio.",
      "Send a test from the Resend UI. Studio never sends mail.",
      "Continue publishing when Publish has an email / notify gate — Confirm after the key is live on the host.",
    ],
  },
];
