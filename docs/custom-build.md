# George's custom EcoPaste build

This fork maintains a small customization series on top of official stable
releases. The initial base is `v1.1.0` (commit
`175126926ea89a2165cc85def0309cd5d27e744d`).

## Repository

- Local checkout: `V:\Repos\EcoPaste`
- `origin`: `https://github.com/george-halachev/EcoPaste-1.git`
- `upstream`: `https://github.com/EcoPasteHub/EcoPaste.git`
- Custom branch: `george-custom`
- Conflict-resolution recording: `git config rerere.enabled true`

Keep custom features in separate commits. Keep upstream `master` free of custom
changes. Drop a custom commit if a later upstream release implements its behavior.

## Windows build

Use Node 20 or newer, the pnpm version pinned in `package.json`, the Rust toolchain
in `rust-toolchain.toml`, Visual Studio C++ Build Tools with a Windows SDK, and
WebView2. From the repository root:

```powershell
pnpm install --frozen-lockfile
pnpm lint
pnpm tsc
node node_modules/@tauri-apps/cli/tauri.js build --config src-tauri/tauri.custom.conf.json --bundles nsis -- --locked
```

The custom overlay marks the build as `1.1.0+1` (custom rebuild 1) and disables creation of
signed updater artifacts. This allows a local installer build without upstream's
private signing key. The EXE and NSIS installer are under
`src-tauri/target/release/` and `src-tauri/target/release/bundle/nsis/`.

Keep build metadata numeric: Windows uses it as the fourth executable version
component. A textual suffix such as `+george.1` is replaced with zero by NSIS.

Install custom builds manually. The existing in-app updater installs official
binaries and will replace custom changes; it does not replay our commits. Turn
off automatic update checks in Preferences > About and use this source upgrade
process. A future custom updater must use our own update feed and signing key;
the update endpoints are also selected in `src-tauri/src/update.rs`.

Development builds use a separate `dev` data directory. The same application
identifier still shares single-instance behavior and global shortcuts: stop the
production app when exercising a native development build, or use a separate
test identifier and disable capture/shortcuts in that test profile.

## Upgrade to another stable release

Start from a clean worktree. Fetch upstream, keep the current branch as a
rollback branch, and replay the custom commits onto the new release. Substitute
the actual new release tag for `<new-release>`:

```powershell
git fetch upstream --tags
git switch george-custom
git branch backup/george-v1.1.0
git switch -c upgrade/<new-release>
git rebase --onto <new-release> v1.1.0
```

Resolve conflicts by reading the new implementation and preserving the intended
behavior, then use `git add` and `git rebase --continue`. Review resolutions
reused by `rerere`; they are suggestions, not proof of correctness. Use
`git rebase --abort` to return the upgrade branch to its original state.

Update the recorded base and custom version in this document and
`src-tauri/tauri.custom.conf.json`. Install dependencies from the new lockfiles,
run checks appropriate to the changed layers, build, and verify cards at narrow
and wide widths, hover actions, keyboard navigation, search, and pasting.

Test any new database migrations with a copy of production data first. After
validation, retain the upgrade branch as the new customization branch or move
`george-custom` to it locally. Push the upgrade branch normally before changing
the published custom branch; rebasing an already published branch requires a
reviewed `--force-with-lease` push.

## Deployment and rollback

Quit EcoPaste before taking a filesystem backup. Read
`%LOCALAPPDATA%\com.ayangweb.eco-paste\prod\storage.json` to identify the active
data directory; back up that directory together with the bootstrap manifest and
the previous EXE/installer. Preserve `db`, `resources`, `config`, and `state`
together. Copying only a live SQLite database file can omit pending WAL data.

Install the verified custom build into the existing installation, start it, and
confirm the clipboard history, settings, and requested behavior. Preserve the
application identifier to keep the existing profile. If rolling back after a
schema change, restore the matching pre-upgrade data backup as well as the old
binary; do not open a newer schema with an older binary unless upstream supports
that downgrade.

The installed NSIS payload has Tauri's `__TAURI_BUNDLE_TYPE_VAR_NSS` marker;
the standalone build EXE has `__TAURI_BUNDLE_TYPE_VAR_UNK`. Their raw SHA-256
hashes differ even when the remaining executable bytes are identical.

## Initial customization

`ClipboardCard` keeps the source-app icon and content in its flexible left
column. Its right column shows the existing translated type label above
`ClipboardQuickActions`, which shows the date at rest and action buttons on
hover. Capture, storage, and database contracts are unchanged.
