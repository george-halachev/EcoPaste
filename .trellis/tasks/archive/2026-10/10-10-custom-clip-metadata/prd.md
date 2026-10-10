# Move clip type above the date and maintain custom Windows builds

## Goal

Maintain George's custom Windows build across upstream releases and move clip type labels into the right-hand metadata column.

## Requirements

- Keep the fork in `V:\Repos\EcoPaste`, with George's GitHub fork as `origin` and the official project as `upstream`.
- Base the first build on stable release `v1.1.0` and keep customizations in focused commits.
- Right-align each translated clip type above its date; keep the source application icon on the left.
- Preserve content, notes, selection, keyboard hints, and hover quick actions.
- Document building, upgrading, backing up, and rolling back the custom version.

## Acceptance Criteria

- [x] Text, image, path, and file types appear above their dates at wide and minimum window widths.
- [x] Hover actions remain usable below the type label.
- [x] Frontend lint and TypeScript checks pass, and the Windows installer builds.
- [x] Inspect the rebuilt native app through the affected list path.
- [x] Commit and push the completed customizations to George's fork.

## Notes

- George approved the fork/upgrade plan and explicitly requested implementation on 2026-10-10.
- No storage quota change is needed: the 1 GB / 2 GB display thresholds do not enforce a limit.
- Validation: actual React card components rendered at 920px and 360px; type/date alignment, overflow, hover copy dispatch, long dates, and note transitions passed. Native installed build `1.1.0+1` showed the changed text, image, path, and files cards.
- Deployment: old installation and the complete active data directory were backed up before running the NSIS installer. All 6,163 history rows and all settings were preserved.
