import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";

export const baseOptions = (): BaseLayoutProps => ({
  nav: {
    title: (
      <span className="flex items-center gap-2">
        <span className="flex size-7 items-center justify-center rounded-md bg-white p-1 shadow-sm ring-1 ring-black/10">
          <img
            alt=""
            aria-hidden="true"
            className="h-full w-full object-contain"
            src="/flexweave.svg"
          />
        </span>
        <span>Flexweave</span>
      </span>
    ),
    url: "/docs",
  },
});
