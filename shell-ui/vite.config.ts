import { defineConfig, type IndexHtmlTransformContext, type Plugin } from "vite";
import react from "@vitejs/plugin-react";

const KIOSK_CSP =
  "default-src 'none'; script-src empiier:; style-src empiier:; img-src empiier: data:; connect-src 'none'; object-src 'none'; frame-src 'none'; base-uri 'none'; form-action 'none'";

/** Vite HMR only. Never written into `dist/`. */
const LOCAL_DEV_CSP =
  "default-src 'none'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self' ws: wss:; font-src 'self'; object-src 'none'; frame-src 'none'; base-uri 'none'; form-action 'none'";

function rewriteCsp(html: string, csp: string): string {
  const next = html.replace(
    /(<meta\b[^>]*http-equiv="Content-Security-Policy"[^>]*content=")([^"]*)(")/,
    `$1${csp}$3`,
  );
  if (!next.includes(csp)) {
    throw new Error("local CSP rewrite missed the kiosk meta");
  }
  return next;
}

const localStylesheet: Plugin = {
  name: "empiieros-local-stylesheet",
  transformIndexHtml: {
    order: "post",
    handler(html: string): string {
      return html.replace(
        /(<link\b[^>]*\brel="stylesheet")\s+crossorigin(?=[\s>])/g,
        "$1",
      );
    },
  },
};

const localBrowser: Plugin = {
  name: "empiieros-local-browser",
  transformIndexHtml: {
    order: "pre",
    handler(html: string, ctx: IndexHtmlTransformContext): string {
      if (!ctx.server) {
        return html;
      }
      if (!html.includes(KIOSK_CSP)) {
        throw new Error("source index.html lost kiosk CSP");
      }
      return rewriteCsp(html, LOCAL_DEV_CSP);
    },
  },
};

export default defineConfig({
  base: "./",
  plugins: [react(), localBrowser, localStylesheet],
  build: {
    sourcemap: false,
    target: "es2022",
  },
  server: {
    host: "127.0.0.1",
    port: 5173,
    strictPort: true,
  },
});
