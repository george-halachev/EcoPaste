# Design

Use two repo-native SVGs on a 24px grid with rounded geometry. The build script
renders PNGs and multi-resolution ICOs with Tauri's SVG renderer. The bundled
executable uses a neutral-gray outline icon that contrasts on light and dark
surfaces; runtime surfaces use light/dark monochrome palettes.

Persist `appearance.iconStyle` as `outline | solid` in Rust's serde-defaulted
Appearance model and mirror the enum in TypeScript. Existing update_settings
and settings://updated carry the setting; no extra frontend persistence.

A Rust app_icon module owns native images and refresh: startup, settings/reset,
window ready/show, and system theme events. Window icons follow the window's
theme; Windows tray/taskbar follow Windows system surface theme. macOS tray is
a template image and Dock icon changes through NSApplication.

Windows shell integration writes stable per-variant ICOs under the bootstrap
state directory, sets explicit AppUserModelID and relaunch icon properties, and
updates only existing shortcuts whose resolved target is this exact installed
EXE. Notify Explorer of those changed shortcuts. Embedded EXE resources remain
the build-time outline default; live windows and launch shortcuts reflect the
saved selection. Do not patch a running binary or reset the global icon cache.

An AppIcon React component uses currentColor SVG masks and the settings mirror.
Replace every application-logo rendering, including the clipboard fallback.
Keep icons identifying other source applications unchanged.

Deployment uses a pre-install profile/application backup. Rollback restores the
old binary and matching profile if needed. No database schema changes.
