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
    tsconfigPaths: true,
  },
  server: {
    port: 3000,
  },
});
