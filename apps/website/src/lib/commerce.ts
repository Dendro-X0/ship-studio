/** Polar + download env (PUBLIC_* inlined by Astro). */

export function polarCheckoutUrl(): string {
  return (import.meta.env.PUBLIC_POLAR_CHECKOUT_URL ?? "").trim();
}

export function polarPortalUrl(): string {
  return (import.meta.env.PUBLIC_POLAR_PORTAL_URL ?? "https://polar.sh/").trim();
}

export function downloadUrl(): string {
  return (
    import.meta.env.PUBLIC_DOWNLOAD_URL ??
    "https://github.com/Dendro-X0/ship-studio/releases"
  ).trim();
}

export function priceLabel(): string {
  return (import.meta.env.PUBLIC_POLAR_PRICE_LABEL ?? "Solo").trim();
}

export function supportEmail(): string {
  return (import.meta.env.PUBLIC_SUPPORT_EMAIL ?? "").trim();
}

export function checkoutConfigured(): boolean {
  return polarCheckoutUrl().length > 0;
}

export function refundWindowDays(): number {
  const raw = (import.meta.env.PUBLIC_REFUND_WINDOW_DAYS ?? "14").trim();
  const n = Number.parseInt(raw, 10);
  return Number.isFinite(n) && n > 0 ? n : 14;
}

/** Paths Polar success/cancel redirects should target (set in Polar dashboard). */
export const CHECKOUT_SUCCESS_PATH = "/checkout/success";
export const CHECKOUT_CANCEL_PATH = "/checkout/cancel";
