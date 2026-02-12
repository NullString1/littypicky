import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import { VitePWA } from "vite-plugin-pwa";

export default defineConfig(() => {
  return {
    plugins: [
      sveltekit(),
      VitePWA({
        registerType: "autoUpdate",
        devOptions: {
          enabled: true,
        },
        includeAssets: ["favicon.ico", "apple-touch-icon.png", "favicon.svg"],
        manifest: {
          name: "LittyPicky",
          short_name: "LittyPicky",
          description: "LittyPicky App",
          theme_color: "#0b172a",
          background_color: "#0b172a",
          icons: [
            {
              src: "pwa-icon-192.png",
              sizes: "192x192",
              type: "image/png",
            },
            {
              src: "pwa-icon-512.png",
              sizes: "512x512",
              type: "image/png",
            },
            {
              src: "pwa-icon-512-maskable.png",
              sizes: "512x512",
              type: "image/png",
              purpose: "any maskable",
            },
          ],
        },
      }),
    ],
    server: {
      proxy: {
        "/api": {
          target: "http://localhost:6780",
          changeOrigin: true,
        },
      },
    },
  };
});
