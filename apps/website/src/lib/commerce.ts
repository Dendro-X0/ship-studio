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

export function checkoutConfigured(): boolean {
  return polarCheckoutUrl().length > 0;
}
