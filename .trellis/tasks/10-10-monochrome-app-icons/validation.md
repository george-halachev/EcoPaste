# Validation

- Frontend: `pnpm lint`, `pnpm tsc`, and production Vite build passed.
- Rust: fmt, locked all-target Clippy with warnings denied, and locked library
  tests passed: 175 passed, 7 existing ignored, no failures.
- Browser: actual AppIcon, preference schema/control, and setting commit path
  exercised in light/dark themes at 16/24/48/96px. Four switches generated only
  the expected appearance.iconStyle patch. English and Chinese labels rendered;
  no page errors. Previews use synthetic content.
- Windows release `1.1.0+3` built and installed. Actual preference UI switched
  outline -> solid, immediately changing branding and native title-bar icon.
  Restarted with solid saved, recreated both clipboard/preferences windows, and
  observed solid retained. Switched back to outline and left the setting open.
- Read back both existing Start menu and Startup shortcuts: icon paths changed
  to solid-neutral.ico, then outline-neutral.ico. No native icon errors logged.
  Tray and taskbar refresh paths use the shared selection and native APIs;
  a separate screenshot of those Windows surfaces was not captured.
- All 6,173 pre-install clipboard IDs remained. Every unrelated setting matched
  the stopped-app backup. Installed EXE matches the release build after
  normalizing Tauri's installer bundle marker. Previous binary/profile backed up.
- macOS API signatures reviewed from locked objc2 sources; Mac runtime/build
  verification remains unperformed on this Windows machine.

# Debug findings

- Real COM shortcut test exposed mutation after STGM_READ as Access Denied.
  Read identity first, then reload exact-target matches with STGM_READWRITE.
  This rule is captured in the platform spec and the regression test.
- A Vite browser fixture cannot assume named exports from prebundled CommonJS
  React modules; use the default export. Importing Valtio through a separately
  addressed prebundle can split module identity. Test the actual setting commit
  path with a mirrored fixture state and real AppIcon's ordinary module imports.
- Keep the fixture Vite process attached to a PTY and restart after source
  changes if esbuild reports its service stopped. Fixture-only utility classes
  require explicit CSS because UnoCSS cannot scan intercepted script content.
