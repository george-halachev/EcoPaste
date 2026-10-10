# Implementation and validation

1. Add SVG sources and deterministic PNG/ICO build generation, inspect previews
   at 16/24px and large sizes in both contrast palettes.
2. Extend Rust/TypeScript Appearance with the serde-defaulted iconStyle enum;
   add bilingual setting labels, segmented choices, and visual mapping.
3. Implement native window/tray/Dock and Windows shell icon refresh, including
   startup, recreated windows, settings reset, and theme changes.
4. Replace app-logo usages with a shared reactive AppIcon.
5. Test old/new/invalid settings and native image decoding. Run pnpm lint,
   pnpm tsc, cargo fmt, cargo clippy and cargo test where appropriate.
6. Build 1.1.0+3; verify the actual UI with synthetic browser fixtures and the
   installed application. Check both setting choices and persistence.
7. Back up and deploy; compare history IDs and settings except the new field.
8. Record native icon contract in specs; commit and push george-custom.
