# Monochrome app icons with a shared style setting

## Requirements

- Create recognizable monochrome EcoPaste icons in Fluent-inspired outline and
  solid styles, preserving the P motif and transparent background.
- Add an Icon style option in Appearance with both choices.
- Apply the saved style to app UI, tray, taskbar, title bars, and app shortcuts.
- Apply changes live and after restarting or rebuilding a window.
- Retain existing history and all unrelated preferences.

## Acceptance criteria

- Both styles look clear at tray and larger sizes in light and dark themes.
- The setting persists and refreshes existing and recreated windows and tray.
- Existing EcoPaste desktop, Start menu, and pinned taskbar shortcuts use the
  selected style without touching shortcuts for other applications.
- Existing settings without iconStyle load with an outline default; invalid
  values are rejected without rewriting valid settings.
- Frontend checks, appropriate Rust checks, release build, and UI verification
  pass; deployment preserves the profile.
