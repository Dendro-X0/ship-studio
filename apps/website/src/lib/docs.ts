/** Public docs routes ← content collection entry ids (glob from repo docs/). */
export type DocPage = {
  slug: string;
  title: string;
  description: string;
  /** Content collection id from Astro glob loader (slugified) */
  entryId: string;
};

export const DOC_PAGES: DocPage[] = [
  {
    slug: "start",
    title: "Start here",
    description: "Boot order, Publish spine, and first commands.",
    entryId: "start-here",
  },
  {
    slug: "scope",
    title: "Scope of service",
    description: "What Ship Studio does and does not do.",
    entryId: "product/scope-of-service",
  },
  {
    slug: "product",
    title: "Product contract",
    description: "Commands, surfaces, and providers.",
    entryId: "product/product",
  },
  {
    slug: "human-gates",
    title: "Human gates",
    description: "Paste, graduate, deploy — still on the operator.",
    entryId: "product/operator-next",
  },
  {
    slug: "current",
    title: "Current status",
    description: "Version, idle/active queue, and truth pointers.",
    entryId: "current",
  },
];

export function docBySlug(slug: string): DocPage | undefined {
  return DOC_PAGES.find((d) => d.slug === slug);
}

/** Rewrite in-repo markdown links to public /docs routes or GitHub. */
export function rewriteDocHref(href: string): string {
  const raw = href.trim();
  if (!raw || raw.startsWith("#") || raw.startsWith("http") || raw.startsWith("/")) {
    return raw;
  }

  const map: Record<string, string> = {
    "./SCOPE-OF-SERVICE.md": "/docs/scope",
    "SCOPE-OF-SERVICE.md": "/docs/scope",
    "./product/SCOPE-OF-SERVICE.md": "/docs/scope",
    "product/SCOPE-OF-SERVICE.md": "/docs/scope",
    "./PRODUCT.md": "/docs/product",
    "PRODUCT.md": "/docs/product",
    "./product/PRODUCT.md": "/docs/product",
    "product/PRODUCT.md": "/docs/product",
    "./OPERATOR-NEXT.md": "/docs/human-gates",
    "OPERATOR-NEXT.md": "/docs/human-gates",
    "./product/OPERATOR-NEXT.md": "/docs/human-gates",
    "product/OPERATOR-NEXT.md": "/docs/human-gates",
    "./START-HERE.md": "/docs/start",
    "../START-HERE.md": "/docs/start",
    "START-HERE.md": "/docs/start",
    "./CURRENT.md": "/docs/current",
    "../CURRENT.md": "/docs/current",
    "CURRENT.md": "/docs/current",
    "./README.md": "/docs",
    "../README.md": "/docs",
    "./handoffs/current-session.md":
      "https://github.com/Dendro-X0/ship-studio/blob/main/docs/handoffs/current-session.md",
    "../handoffs/current-session.md":
      "https://github.com/Dendro-X0/ship-studio/blob/main/docs/handoffs/current-session.md",
  };

  if (map[raw]) return map[raw];

  if (raw.includes("shipping-hub-north-star")) {
    return "https://github.com/Dendro-X0/ship-studio/blob/main/specs/backend/shipping-hub-north-star.md";
  }
  if (raw.includes("release-surface-map")) {
    return "https://github.com/Dendro-X0/ship-studio/blob/main/specs/backend/release-surface-map.md";
  }
  if (raw.includes("product-website-charter")) {
    return "https://github.com/Dendro-X0/ship-studio/blob/main/specs/backend/product-website-charter.md";
  }
  if (raw.includes("guided-launch-design") || raw.includes("studio-modes-design")) {
    const file = raw.split("/").pop() ?? raw;
    return `https://github.com/Dendro-X0/ship-studio/blob/main/specs/backend/${file}`;
  }
  if (raw.includes("specs/backend/")) {
    const idx = raw.indexOf("specs/backend/");
    return `https://github.com/Dendro-X0/ship-studio/blob/main/${raw.slice(idx)}`;
  }

  return raw;
}
