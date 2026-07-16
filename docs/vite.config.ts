import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import tailwindcss from "@tailwindcss/vite";
import mdx from "fumadocs-mdx/vite";
import { nitro } from "nitro/vite";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import viteReact from "@vitejs/plugin-react";

const publicDir = fileURLToPath(new URL("../assets", import.meta.url));

export default defineConfig({
  plugins: [
    mdx(),
    tailwindcss(),
    tanstackStart(),
    viteReact(),
    nitro({ publicAssets: [{ dir: publicDir, maxAge: 0 }] }),
  ],
  publicDir,
  resolve: {
    // Keep the Nitro bundle self-contained while avoiding Rolldown's CommonJS
    // default-import handling for tslib. The second alias prevents the adapter's
    // explicit ESM import from resolving back to itself.
    alias: [
      {
        find: /^tslib$/,
        replacement: fileURLToPath(new URL("src/lib/tslib.mjs", import.meta.url)),
      },
      {
        find: /^tslib\/tslib\.es6\.mjs$/,
        replacement: fileURLToPath(new URL("node_modules/tslib/tslib.es6.mjs", import.meta.url)),
      },
    ],
    tsconfigPaths: true,
  },
  server: {
    port: 3000,
  },
});
