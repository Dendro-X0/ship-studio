import { defineCollection, z } from "astro:content";
import { glob } from "astro/loaders";

/** Repo docs rooted at ship-studio/docs (sibling of apps/). */
const docs = defineCollection({
  loader: glob({
    pattern: "{START-HERE.md,CURRENT.md,product/SCOPE-OF-SERVICE.md,product/PRODUCT.md,product/OPERATOR-NEXT.md}",
    base: "../../docs",
  }),
  schema: z.object({}).passthrough(),
});

export const collections = { docs };
