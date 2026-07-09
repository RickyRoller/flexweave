import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import tailwindcss from "@tailwindcss/vite";
import mdx from "fumadocs-mdx/vite";
import { nitro } from "nitro/vite";
import { defineConfig } from "vite";
import viteReact from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [mdx(), tailwindcss(), tanstackStart(), viteReact(), nitro()],
  resolve: {
    tsconfigPaths: true,
  },
  server: {
    port: 3000,
  },
});
