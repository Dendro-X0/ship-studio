/** Suite shelf — honest portfolio links (not Integrations theater). */

export type SuiteCard = {
  id: string;
  title: string;
  blurb: string;
  /** Primary open URL, if public. */
  openUrl?: string;
  openLabel?: string;
  /** Secondary URL (docs / GitHub). */
  secondaryUrl?: string;
  secondaryLabel?: string;
  note?: string;
};

/**
 * Public demos / OSS where available. Velocity stays private — no fake storefront link.
 * Update URLs when suite sites move; do not invent paywalls here.
 */
export const SUITE_CARDS: SuiteCard[] = [
  {
    id: "strata",
    title: "Strata",
    blurb:
      "Whole-kit SaaS starter — polished demo for “take the kit” clients. Separate product; not required to Get live here.",
    openUrl: "https://strata-web-ivory.vercel.app/",
    openLabel: "Open demo",
    note: "Optional scaffold · keys stay on your host",
  },
  {
    id: "velocity",
    title: "Velocity",
    blurb:
      "Modular scaffold — compose auth, payments, email adapters into a repo. You still put API keys on the vendor; or hire a wall if you want it stood up.",
    note: "Private suite tool · no fake public buy link",
  },
  {
    id: "truss",
    title: "Truss",
    blurb:
      "Lightweight multi-tenant OSS starter — same DNA as Strata, smaller footprint.",
    openUrl: "https://github.com/Dendro-X0/Truss",
    openLabel: "GitHub",
    secondaryUrl: "https://truss-web-omega.vercel.app/",
    secondaryLabel: "Live demo",
  },
];
