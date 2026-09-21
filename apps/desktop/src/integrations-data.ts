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
      "Create a one-time product on Polar.",
      "Set success URL to /checkout/success and cancel to /checkout/cancel.",
      "Copy the checkout link and customer portal URL.",
      "Paste them into the deploy env or apps/website PUBLIC_POLAR_* — Studio does not write those.",
      "Refunds stay on the Polar portal within your published window.",
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
      "Open the Stripe dashboard.",
      "Create or update the product and checkout yourself.",
      "Copy any publishable or secret names you need — paste values only on the deploy host.",
      "Return and confirm the listing in Publish when that step is current.",
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
      "Copy the product URL if your site or listing needs it.",
      "Confirm in Publish when the Gumroad listing step is current.",
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
      "Open Lemon Squeezy.",
      "Create or update the product and checkout.",
      "Confirm fulfillment email works in Lemon — Studio does not send it.",
      "Confirm the listing step in Publish when it is current.",
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
      "Open the Paddle vendor dashboard.",
      "Create or update the product and checkout yourself.",
      "Confirm the listing step in Publish when it is current.",
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
      "Open Resend API keys and create a key.",
      "Put RESEND_API_KEY on the deploy host from Env — do not paste the value into Studio.",
      "Send a test from Resend. Studio never sends mail.",
    ],
  },
];
