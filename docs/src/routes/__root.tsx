/// <reference types="vite/client" />

import { createRootRoute, HeadContent, Outlet, Scripts } from "@tanstack/react-router";
import { RootProvider } from "fumadocs-ui/provider/tanstack";
import type { ReactNode } from "react";
import "../styles/app.css";

const RootDocument = ({ children }: Readonly<{ children: ReactNode }>) => (
  <html lang="en" suppressHydrationWarning>
    <head>
      <HeadContent />
    </head>
    <body className="flex min-h-screen flex-col">
      <RootProvider>{children}</RootProvider>
      <Scripts />
    </body>
  </html>
);

const RootComponent = () => (
  <RootDocument>
    <Outlet />
  </RootDocument>
);

export const Route = createRootRoute({
  component: RootComponent,
  head: () => ({
    links: [
      {
        href: "/flexweave.svg",
        rel: "icon",
        type: "image/svg+xml",
      },
    ],
    meta: [
      {
        charSet: "utf-8",
      },
      {
        content: "width=device-width, initial-scale=1",
        name: "viewport",
      },
      {
        title: "Flexweave Documentation",
      },
      {
        content: "Documentation for the Flexweave mechanics primitive crate.",
        name: "description",
      },
    ],
  }),
});
