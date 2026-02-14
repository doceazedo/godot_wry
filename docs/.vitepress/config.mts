import { defineConfig } from "vitepress";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Godot WRY",
  description: "Cross-platform webview extension for Godot 4",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: "Guide", link: "/guide/what-is-godot-wry" },
      { text: "Reference", link: "/reference/webview" },
      { text: "Examples", link: "/examples/character-creation-ui" },
    ],

    sidebar: [
      {
        text: "Introduction",
        items: [
          { text: "What is Godot WRY?", link: "/guide/what-is-godot-wry" },
          { text: "Getting started", link: "/guide/getting-started" },
          {
            text: "Source code",
            link: "https://github.com/doceazedo/godot_wry",
          },
          {
            text: "Asset Library",
            link: "https://godotengine.org/asset-library/asset/3426",
          },
        ],
      },
      {
        text: "Examples",
        items: [
          {
            text: "Character creation UI",
            link: "/examples/character-creation-ui",
          },
        ],
      },
      {
        text: "About",
        items: [
          { text: "Roadmap", link: "/about/roadmap" },
          { text: "Caveats", link: "/about/caveats" },
          { text: "Alternatives", link: "/about/alternatives" },
        ],
      },
      {
        text: "Manual",
        items: [
          { text: "Interoperability", link: "/tutorials/gdscript-js-interop" },
          { text: "Exporting your project", link: "/tutorials/exporting" },
        ],
      },
      {
        text: "Contributing",
        items: [
          {
            text: "How to contribute",
            link: "/contributing/how-to-contribute",
          },
          { text: "Building from source", link: "/contributing/compiling" },
          { text: "Writing documentation", link: "/contributing/docs" },
        ],
      },
      {
        text: "API Reference",
        items: [
          { text: "WebView", link: "/reference/webview" },
          { text: "JavaScript", link: "/reference/javascript" },
        ],
      },
    ],

    socialLinks: [
      { icon: "github", link: "https://github.com/doceazedo/godot_wry" },
      { icon: "discord", link: "https://discord.gg/B9fWw3raZJ" },
    ],

    logo: { src: "/logo.svg", width: 24, height: 24 },

    footer: {
      message: "Released under the MIT license.",
      copyright: "© 2026 Doce Fernandes & Godot WRY contributors",
    },

    externalLinkIcon: true,

    editLink: {
      pattern: "https://github.com/doceazedo/godot_wry/edit/main/docs/:path",
    },

    search: {
      provider: "local",
    },
    outline: [2, 3],
  },
  head: [["link", { rel: "icon", href: "/favicon.png" }]],
  ignoreDeadLinks: true,
});
