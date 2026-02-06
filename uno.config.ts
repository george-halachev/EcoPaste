import {
  defineConfig,
  presetIcons,
  presetUno,
  transformerDirectives,
  transformerVariantGroup,
} from "unocss";

export default defineConfig({
  presets: [
    presetUno(),
    presetIcons({
      collections: {
        hugeicons: () =>
          import("@iconify-json/hugeicons/icons.json").then((m) => m.default),
        iconamoon: () =>
          import("@iconify-json/iconamoon/icons.json").then((m) => m.default),
        "lets-icons": () =>
          import("@iconify-json/lets-icons/icons.json").then((m) => m.default),
        logos: () =>
          import("@iconify-json/logos/icons.json").then((m) => m.default),
        lucide: () =>
          import("@iconify-json/lucide/icons.json").then((m) => m.default),
        "simple-icons": () =>
          import("@iconify-json/simple-icons/icons.json").then(
            (m) => m.default,
          ),
        "skill-icons": () =>
          import("@iconify-json/skill-icons/icons.json").then((m) => m.default),
      },
    }),
  ],
  rules: [["outline-none", { outline: "none" }]],
  safelist: [
    "i-hugeicons:database-export",
    "i-hugeicons:database-import",
    "i-hugeicons:task-edit-01",
    "i-iconamoon:close-circle-1",
    "i-iconamoon:star",
    "i-iconamoon:star-fill",
    "i-iconamoon:volume-up-light",
    "i-lets-icons:pin",
    "i-lets-icons:setting-alt-line",
    "i-lucide:bolt",
    "i-lucide:circle-arrow-right",
    "i-lucide:circle-check",
    "i-lucide:clipboard-list",
    "i-lucide:clipboard-paste",
    "i-lucide:clipboard-pen-line",
    "i-lucide:copy",
    "i-lucide:history",
    "i-lucide:info",
    "i-lucide:keyboard",
    "i-lucide:search",
    "i-lucide:trash",
  ],
  shortcuts: [
    [/^bg-color-(\d+)$/, ([, d]) => `bg-bg-${d}`],
    [/^text-color-(\d+)$/, ([, d]) => `text-text-${d}`],
    [/^b-color-(\d+)$/, ([, d]) => `b-border-${d}`],
    [/^(.*)-primary-(\d+)$/, ([, s, d]) => `${s}-[var(--ant-blue-${d})]`],
  ],
  theme: {
    colors: {
      alipay: "#0c79fe",
      "bg-1": "var(--ant-color-bg-container)",
      "bg-2": "var(--ant-color-bg-layout)",
      "bg-3": "var(--ant-color-fill-quaternary)",
      "bg-4": "var(--ant-color-fill-content)",
      "border-1": "var(--ant-color-border)",
      "border-2": "var(--ant-color-border-secondary)",
      danger: "var(--ant-red)",
      gold: "var(--ant-gold)",
      primary: "var(--ant-blue)",
      qq: "#0099ff",
      success: "var(--ant-green)",
      "text-1": "var(--ant-color-text)",
      "text-2": "var(--ant-color-text-secondary)",
      "text-3": "var(--ant-color-text-tertiary)",
      wechat: "#00c25f",
    },
  },
  transformers: [
    transformerVariantGroup(),
    transformerDirectives({
      applyVariable: ["--uno"],
    }),
  ],
});
