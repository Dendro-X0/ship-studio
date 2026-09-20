import { defineConfig } from "astro/config";
import { visit } from "unist-util-visit";
import { rewriteDocHref } from "./src/lib/docs.ts";

function rehypeRewriteDocLinks() {
  return (tree) => {
    visit(tree, "element", (node) => {
      if (node.tagName !== "a" || !node.properties?.href) return;
      node.properties.href = rewriteDocHref(String(node.properties.href));
    });
  };
}

export default defineConfig({
  site: "https://shipstudio.dev",
  output: "static",
  trailingSlash: "never",
  vite: {
    server: {
      fs: {
        allow: ["../.."],
      },
    },
  },
  markdown: {
    rehypePlugins: [rehypeRewriteDocLinks],
  },
});
