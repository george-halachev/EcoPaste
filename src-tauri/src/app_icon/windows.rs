use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use anyhow::Context;
use tauri::{AppHandle, Manager, WebviewWindow};
use windows::core::{ComInterface, GUID, PCWSTR, PWSTR};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, IPersistFile,
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, STGM_READ, STGM_READWRITE,
};
use windows::Win32::System::Registry::{
    RegCloseKey, RegNotifyChangeKeyValue, RegOpenKeyExW, HKEY, HKEY_CURRENT_USER, KEY_NOTIFY,
    REG_NOTIFY_CHANGE_LAST_SET,
};
use windows::Win32::System::Variant::VT_LPWSTR;
use windows::Win32::UI::Shell::PropertiesSystem::{
    IPropertyStore, SHGetPropertyStoreForWindow, PROPERTYKEY,
};
use windows::Win32::UI::Shell::{
    FOLDERID_CommonPrograms, FOLDERID_Desktop, FOLDERID_Programs, FOLDERID_PublicDesktop,
    FOLDERID_QuickLaunch, IShellLinkW, SHChangeNotify, SHGetKnownFolderPath, ShellLink,
    KF_FLAG_DEFAULT, SHCNE_UPDATEITEM, SHCNF_PATHW, SLGP_RAWPATH,
};

use crate::core::Result;
use crate::settings::{IconStyle, SettingsStore};

static SHORTCUT_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

struct RegistryKey(HKEY);
impl Drop for RegistryKey {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

/// A blocking registry notification watches only theme changes, without polling.
pub fn watch_system_theme(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let result = (|| -> Result<()> {
            let name = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
            let mut key = HKEY::default();
            unsafe {
                RegOpenKeyExW(
                    HKEY_CURRENT_USER,
                    PCWSTR(name.as_ptr()),
                    0,
                    KEY_NOTIFY,
                    &mut key,
                )
                .context("watch shell theme registry")?;
            }
            let key = RegistryKey(key);
            let mut dark = shell_is_dark();
            loop {
                unsafe {
                    RegNotifyChangeKeyValue(key.0, false, REG_NOTIFY_CHANGE_LAST_SET, None, false)
                        .context("wait for shell theme change")?;
                }
                let next = shell_is_dark();
                if next != dark {
                    dark = next;
                    super::apply(&app, &app.state::<SettingsStore>().snapshot());
                }
            }
        })();
        if let Err(err) = result {
            log::warn!("watch shell theme: {err}");
        }
    });
}
const APP_MODEL_FMTID: GUID = GUID::from_u128(0x9f4c2855_9f79_4b39_a8d0_e1d42de1d5f3);
const APP_ID: PROPERTYKEY = PROPERTYKEY {
    fmtid: APP_MODEL_FMTID,
    pid: 5,
};
const RELAUNCH_COMMAND: PROPERTYKEY = PROPERTYKEY {
    fmtid: APP_MODEL_FMTID,
    pid: 2,
};
const RELAUNCH_ICON: PROPERTYKEY = PROPERTYKEY {
    fmtid: APP_MODEL_FMTID,
    pid: 3,
};
const RELAUNCH_NAME: PROPERTYKEY = PROPERTYKEY {
    fmtid: APP_MODEL_FMTID,
    pid: 4,
};

pub fn shell_is_dark() -> bool {
    windows_registry::CURRENT_USER
        .open("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")
        .and_then(|key| key.get_u32("SystemUsesLightTheme"))
        .map(|light| light == 0)
        .unwrap_or(true)
}

/// Uses the fixed bootstrap location so custom data-directory moves don't break pins.
fn icon_path(app: &AppHandle, style: IconStyle, palette: &str) -> Result<PathBuf> {
    let (name, bytes): (&str, &[u8]) = match (style, palette) {
        (IconStyle::Outline, "dark") => (
            "outline-dark",
            include_bytes!("../../assets/app-icons/outline-dark.ico"),
        ),
        (IconStyle::Outline, "light") => (
            "outline-light",
            include_bytes!("../../assets/app-icons/outline-light.ico"),
        ),
        (IconStyle::Solid, "dark") => (
            "solid-dark",
            include_bytes!("../../assets/app-icons/solid-dark.ico"),
        ),
        (IconStyle::Solid, "light") => (
            "solid-light",
            include_bytes!("../../assets/app-icons/solid-light.ico"),
        ),
        (IconStyle::Outline, _) => (
            "outline-neutral",
            include_bytes!("../../assets/app-icons/outline-neutral.ico"),
        ),
        (IconStyle::Solid, _) => (
            "solid-neutral",
            include_bytes!("../../assets/app-icons/solid-neutral.ico"),
        ),
    };
    let directory = crate::core::paths::bootstrap_dir(app)?.join("state/app-icons");
    std::fs::create_dir_all(&directory).context("create shell icon directory")?;
    let path = directory.join(format!("{name}.ico"));
    if std::fs::read(&path).ok().as_deref() != Some(bytes) {
        std::fs::write(&path, bytes).context("write shell icon")?;
    }
    Ok(path)
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

/// IPropertyStore copies the string; the borrowed UTF-16 buffer lives through SetValue.
unsafe fn set_string(
    store: &IPropertyStore,
    key: &PROPERTYKEY,
    value: &str,
) -> windows::core::Result<()> {
    let mut text = wide(value);
    let mut variant = PROPVARIANT::default();
    let value = &mut *variant.Anonymous.Anonymous;
    value.vt = VT_LPWSTR;
    value.Anonymous.pwszVal = PWSTR(text.as_mut_ptr());
    store.SetValue(key, &variant)
}

/// Explicit relaunch properties control the icon of newly pinned taskbar groups.
pub fn set_taskbar_identity(
    app: &AppHandle,
    window: &WebviewWindow,
    style: IconStyle,
) -> Result<()> {
    let path = icon_path(app, style, if shell_is_dark() { "dark" } else { "light" })?;
    let hwnd = HWND(window.hwnd().context("get application window handle")?.0 as _);
    let executable = std::env::current_exe().context("resolve running executable")?;
    unsafe {
        let store: IPropertyStore =
            SHGetPropertyStoreForWindow(hwnd).context("get taskbar property store")?;
        set_string(
            &store,
            &RELAUNCH_COMMAND,
            &format!("\"{}\"", executable.display()),
        )
        .context("set relaunch command")?;
        set_string(&store, &RELAUNCH_NAME, "EcoPaste").context("set relaunch name")?;
        set_string(&store, &RELAUNCH_ICON, &format!("{},0", path.display()))
            .context("set taskbar icon")?;
        set_string(&store, &APP_ID, &app.config().identifier)
            .context("set taskbar app identity")?;
    }
    Ok(())
}

struct ComApartment;
impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

/// Refreshes only links whose target matches the current executable exactly.
pub fn refresh_shortcuts(app: &AppHandle) {
    let Ok(_lock) = SHORTCUT_LOCK.lock() else {
        return;
    };
    let result = (|| -> Result<()> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED)
                .context("initialize shortcut COM apartment")?;
        }
        let _apartment = ComApartment;
        let style = app
            .state::<SettingsStore>()
            .snapshot()
            .appearance
            .icon_style;
        let icon = icon_path(app, style, "neutral")?;
        let executable = std::env::current_exe().context("resolve executable for shortcuts")?;
        let identifier = &app.config().identifier;
        for folder in [
            FOLDERID_Desktop,
            FOLDERID_PublicDesktop,
            FOLDERID_Programs,
            FOLDERID_CommonPrograms,
            FOLDERID_QuickLaunch,
        ] {
            let root = unsafe {
                let Ok(value) = SHGetKnownFolderPath(&folder, KF_FLAG_DEFAULT, None) else {
                    continue;
                };
                let path = value.to_string();
                CoTaskMemFree(Some(value.0.cast()));
                PathBuf::from(path.context("read shortcut folder")?)
            };
            for entry in walkdir::WalkDir::new(root)
                .max_depth(5)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if !entry.file_type().is_file()
                    || path
                        .extension()
                        .is_none_or(|ext| !ext.eq_ignore_ascii_case("lnk"))
                {
                    continue;
                }
                if let Err(err) = update_shortcut(path, &executable, &icon, identifier) {
                    log::warn!("update app shortcut {}: {err}", path.display());
                }
            }
        }
        Ok(())
    })();
    if let Err(err) = result {
        log::warn!("refresh application shortcuts: {err}");
    }
}

fn update_shortcut(path: &Path, executable: &Path, icon: &Path, identifier: &str) -> Result<()> {
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
            .context("create shell link reader")?;
        let file: IPersistFile = link.cast().context("get shell link persistence")?;
        let filename = wide(&path.to_string_lossy());
        file.Load(PCWSTR(filename.as_ptr()), STGM_READ)
            .context("load shortcut")?;
        let mut target = [0_u16; 32768];
        link.GetPath(&mut target, std::ptr::null_mut(), SLGP_RAWPATH.0 as u32)
            .context("read shortcut target")?;
        let length = target.iter().position(|c| *c == 0).unwrap_or(target.len());
        let target = PathBuf::from(String::from_utf16_lossy(&target[..length]));
        if !same_executable(&target, executable) {
            return Ok(());
        }
        file.Load(PCWSTR(filename.as_ptr()), STGM_READWRITE)
            .context("open matching shortcut for icon update")?;
        let mut current_icon = [0_u16; 32768];
        let mut index = 0;
        link.GetIconLocation(&mut current_icon, &mut index)
            .context("read shortcut icon")?;
        let length = current_icon
            .iter()
            .position(|c| *c == 0)
            .unwrap_or(current_icon.len());
        let selected = icon.to_string_lossy();
        if index == 0 && String::from_utf16_lossy(&current_icon[..length]) == selected {
            return Ok(());
        }
        let location = wide(&selected);
        link.SetIconLocation(PCWSTR(location.as_ptr()), 0)
            .context("set shortcut icon")?;
        let properties: IPropertyStore = link.cast().context("get shortcut properties")?;
        set_string(&properties, &APP_ID, identifier).context("set shortcut app identity")?;
        properties.Commit().context("commit shortcut identity")?;
        file.Save(PCWSTR(filename.as_ptr()), true)
            .context("save shortcut icon")?;
        SHChangeNotify(
            SHCNE_UPDATEITEM,
            SHCNF_PATHW,
            Some(filename.as_ptr().cast()),
            None,
        );
    }
    Ok(())
}

fn same_executable(target: &Path, executable: &Path) -> bool {
    let target = target
        .canonicalize()
        .unwrap_or_else(|_| target.to_path_buf());
    let executable = executable
        .canonicalize()
        .unwrap_or_else(|_| executable.to_path_buf());
    target
        .to_string_lossy()
        .eq_ignore_ascii_case(&executable.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_exact_executable_targets_match() {
        let executable = Path::new("C:/Program Files/EcoPaste/EcoPaste.exe");
        assert!(same_executable(
            Path::new("c:/program files/ecopaste/ecopaste.exe"),
            executable
        ));
        assert!(!same_executable(
            Path::new("C:/Other/EcoPaste.exe"),
            executable
        ));
        assert!(!same_executable(
            Path::new("C:/Program Files/EcoPaste/uninstall.exe"),
            executable
        ));
    }

    #[test]
    fn shortcut_refresh_preserves_arguments_and_ignores_other_targets() {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap();
            let _apartment = ComApartment;
            let directory = tempfile::tempdir().unwrap();
            let executable = std::env::current_exe().unwrap();
            let icon = directory.path().join("solid.ico");
            std::fs::write(
                &icon,
                include_bytes!("../../assets/app-icons/solid-neutral.ico"),
            )
            .unwrap();

            for (name, target) in [
                ("app", executable.clone()),
                ("other", directory.path().join("Other.exe")),
            ] {
                let path = directory.path().join(format!("{name}.lnk"));
                let link: IShellLinkW =
                    CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).unwrap();
                let target = wide(&target.to_string_lossy());
                link.SetPath(PCWSTR(target.as_ptr())).unwrap();
                let arguments = wide("--auto-launch");
                link.SetArguments(PCWSTR(arguments.as_ptr())).unwrap();
                let description = wide("Preserve this description");
                link.SetDescription(PCWSTR(description.as_ptr())).unwrap();
                let file: IPersistFile = link.cast().unwrap();
                let filename = wide(&path.to_string_lossy());
                file.Save(PCWSTR(filename.as_ptr()), true).unwrap();
                let before = std::fs::read(&path).unwrap();

                update_shortcut(&path, &executable, &icon, "com.ayangweb.eco-paste").unwrap();
                if name == "other" {
                    assert_eq!(std::fs::read(path).unwrap(), before);
                    continue;
                }

                file.Load(PCWSTR(filename.as_ptr()), STGM_READ).unwrap();
                let mut saved_icon = [0_u16; 32768];
                let mut index = -1;
                link.GetIconLocation(&mut saved_icon, &mut index).unwrap();
                let length = saved_icon.iter().position(|c| *c == 0).unwrap();
                assert_eq!(
                    String::from_utf16_lossy(&saved_icon[..length]),
                    icon.to_string_lossy()
                );
                assert_eq!(index, 0);
                let mut saved_arguments = [0_u16; 128];
                link.GetArguments(&mut saved_arguments).unwrap();
                let length = saved_arguments.iter().position(|c| *c == 0).unwrap();
                assert_eq!(
                    String::from_utf16_lossy(&saved_arguments[..length]),
                    "--auto-launch"
                );
                let mut saved_description = [0_u16; 128];
                link.GetDescription(&mut saved_description).unwrap();
                let length = saved_description.iter().position(|c| *c == 0).unwrap();
                assert_eq!(
                    String::from_utf16_lossy(&saved_description[..length]),
                    "Preserve this description"
                );
            }
        }
    }
}
