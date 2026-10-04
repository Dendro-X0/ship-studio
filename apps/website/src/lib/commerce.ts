/** Paddle overlay (Solo owner) + download env. */

export function paddleClientToken(): string {
  return (import.meta.env.PUBLIC_PADDLE_CLIENT_TOKEN ?? "").trim();
}

export function paddlePriceId(): string {
  return (import.meta.env.PUBLIC_PADDLE_PRICE_ID ?? "").trim();
}

/** `sandbox` (default) or `live`. */
export function paddleEnv(): "sandbox" | "live" {
  const raw = (import.meta.env.PUBLIC_PADDLE_ENV ?? "sandbox").trim().toLowerCase();
  return raw === "live" ? "live" : "sandbox";
}

export function paddlePortalUrl(): string {
  return (import.meta.env.PUBLIC_PADDLE_PORTAL_URL ?? "").trim();
}

export function downloadUrl(): string {
  return (
    import.meta.env.PUBLIC_DOWNLOAD_URL ??
    "https://github.com/Dendro-X0/ship-studio/releases"
  ).trim();
}

export function priceLabel(): string {
  return (import.meta.env.PUBLIC_PADDLE_PRICE_LABEL ?? "Solo").trim();
}

export function supportEmail(): string {
  return (import.meta.env.PUBLIC_SUPPORT_EMAIL ?? "").trim();
}

export function paddleConfigured(): boolean {
  return paddleClientToken().length > 0 && paddlePriceId().length > 0;
}

/** Solo Buy is ready only when Paddle overlay env is set. */
export function checkoutConfigured(): boolean {
  return paddleConfigured();
}

export function refundWindowDays(): number {
  const raw = (import.meta.env.PUBLIC_REFUND_WINDOW_DAYS ?? "14").trim();
  const n = Number.parseInt(raw, 10);
  return Number.isFinite(n) && n > 0 ? n : 14;
}

export const CHECKOUT_SUCCESS_PATH = "/checkout/success";
export const CHECKOUT_CANCEL_PATH = "/checkout/cancel";
