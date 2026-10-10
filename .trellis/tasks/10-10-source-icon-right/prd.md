# Move source icon into clip metadata

## Requirements

- Put the source application icon in the right column beside the clip type.
- Reserve the complete left column for clip content or its note.
- Keep the type above the date and preserve hover actions and shortcut hints.
- Use the renamed fork at https://github.com/george-halachev/EcoPaste.
- Build and install the update while preserving the current profile.

## Acceptance criteria

- Text, path, image, and file cards show the icon and type on the right.
- Content starts at the top of the left column with no reserved icon header.
- Narrow and wide cards have no overflow or metadata/status overlap.
- Hover actions, note transitions, and modifier shortcut hints still work.
- Lint, type checks, release build, and native visual verification pass.
- The installation retains history and settings.

## Validation results

- Lint and TypeScript checks passed.
- Windows release EXE and NSIS installer built as 1.1.0+2.
- Actual React component verified in headless Edge at 920px and 360px with
  synthetic text, path, image, file, long-date, and pinned-note clips.
- Source icon placement, content starting at the top, metadata alignment,
  overflow, status spacing, hover actions, note transitions, and Ctrl+1 hint and
  quick-paste dispatch passed.
- Installer completed successfully. All 6,171 history IDs and the settings JSON
  were preserved. Installed EXE matches the build after normalizing Tauri's
  expected NSIS bundle marker.
- Native visual inspection was interrupted by the user's Escape key before a
  screenshot could be captured; no native visual verification is claimed.

## Goal

TBD.

## Requirements

- TBD

## Acceptance Criteria

- [ ] TBD

## Notes

- Keep `prd.md` focused on requirements, constraints, and acceptance criteria.
- Lightweight tasks can remain PRD-only.
- For complex tasks, add `design.md` for technical design and `implement.md` for execution planning before `task.py start`.
