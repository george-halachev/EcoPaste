//! Shared monochrome application identity for native and shell surfaces.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use anyhow::Context;
use tauri::{image::Image, AppHandle, Manager, WebviewWindow};

use crate::core::Result;
use crate::settings::{IconStyle, Settings, SettingsStore, Theme};

/// Shell theme changes must refresh icons even when the app theme is forced.
pub fn watch_system_theme(app: &AppHandle) {
    #[cfg(target_os = "windows")]
    windows::watch_system_theme(app);
    #[cfg(target_os = "macos")]
    let _ = app;
}

/// Decodes a theme-specific icon without depending on installed resource paths.
pub fn image(style: IconStyle, dark: bool) -> Result<Image<'static>> {
    let png = png_bytes(style, dark);
    let decoded = image::load_from_memory(png)
        .context("decode application icon")?
        .into_rgba8();
    let (width, height) = decoded.dimensions();
    Ok(Image::new_owned(decoded.into_raw(), width, height))
}

fn png_bytes(style: IconStyle, dark: bool) -> &'static [u8] {
    match (style, dark) {
        (IconStyle::Outline, false) => {
            include_bytes!("../../assets/app-icons/outline-light-256.png")
        }
        (IconStyle::Outline, true) => include_bytes!("../../assets/app-icons/outline-dark-256.png"),
        (IconStyle::Solid, false) => include_bytes!("../../assets/app-icons/solid-light-256.png"),
        (IconStyle::Solid, true) => include_bytes!("../../assets/app-icons/solid-dark-256.png"),
    }
}

#[cfg(target_os = "macos")]
fn neutral_png(style: IconStyle) -> &'static [u8] {
    match style {
        IconStyle::Outline => include_bytes!("../../assets/app-icons/outline-neutral-256.png"),
        IconStyle::Solid => include_bytes!("../../assets/app-icons/solid-neutral-256.png"),
    }
}

/// Windows notification area contrast follows the shell, not the app theme.
pub fn tray_image(style: IconStyle) -> Result<Image<'static>> {
    #[cfg(target_os = "windows")]
    let dark = windows::shell_is_dark();
    #[cfg(target_os = "macos")]
    let dark = false;
    image(style, dark)
}

/// Refreshes existing native windows, tray, Dock, and Windows launch shortcuts.
pub fn apply(app: &AppHandle, settings: &Settings) {
    let app_handle = app.clone();
    let appearance = settings.appearance.clone();
    if let Err(err) = app.run_on_main_thread(move || {
        for window in app_handle.webview_windows().values() {
            if let Err(err) = set_window_icon(&app_handle, window, &appearance) {
                log::warn!("refresh window icon for {}: {err}", window.label());
            }
        }
        let tray_result = tray_image(appearance.icon_style)
            .and_then(|icon| crate::tray::set_icon(&app_handle, icon));
        if let Err(err) = tray_result {
            log::warn!("refresh tray icon: {err}");
        }
        #[cfg(target_os = "macos")]
        if let Err(err) = macos::set_dock_icon(neutral_png(appearance.icon_style)) {
            log::warn!("refresh Dock icon: {err}");
        }
    }) {
        log::warn!("schedule application icon refresh: {err}");
    }

    #[cfg(target_os = "windows")]
    {
        // COM and shortcut IO run off the UI thread; serialize writes so a later
        // style selection cannot be overwritten by an older refresh finishing last.
        let app_handle = app.clone();
        std::thread::spawn(move || {
            windows::refresh_shortcuts(&app_handle);
        });
    }
}

/// Applies the saved identity when a lazy window is created or shown again.
pub fn apply_window(app: &AppHandle, label: &str) {
    let Some(store) = app.try_state::<SettingsStore>() else {
        return;
    };
    let appearance = store.snapshot().appearance;
    let app_handle = app.clone();
    let label = label.to_owned();
    if let Err(err) = app.run_on_main_thread(move || {
        if let Some(window) = app_handle.get_webview_window(&label) {
            if let Err(err) = set_window_icon(&app_handle, &window, &appearance) {
                log::warn!("apply icon to {label}: {err}");
            }
        }
    }) {
        log::warn!("schedule window icon refresh: {err}");
    }
}

fn set_window_icon(
    app: &AppHandle,
    window: &WebviewWindow,
    appearance: &crate::settings::Appearance,
) -> Result<()> {
    let dark = if window.label() == crate::window::ONBOARDING_WINDOW_LABEL {
        true
    } else {
        match appearance.theme {
            Theme::Dark => true,
            Theme::Light => false,
            Theme::Auto => window.theme().ok() == Some(tauri::Theme::Dark),
        }
    };
    window
        .set_icon(image(appearance.icon_style, dark)?)
        .context("set native window icon")?;
    #[cfg(target_os = "windows")]
    windows::set_taskbar_identity(app, window, appearance.icon_style)?;
    #[cfg(target_os = "macos")]
    let _ = app;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_styles_decode_with_alpha_and_monochrome_pixels() {
        for style in [IconStyle::Outline, IconStyle::Solid] {
            for dark in [false, true] {
                let icon = image(style, dark).unwrap();
                assert_eq!((icon.width(), icon.height()), (256, 256));
                assert!(icon.rgba().chunks_exact(4).any(|pixel| pixel[3] == 0));
                assert!(icon.rgba().chunks_exact(4).any(|pixel| pixel[3] == 255));
                assert!(icon
                    .rgba()
                    .chunks_exact(4)
                    .all(|p| p[0] == p[1] && p[1] == p[2]));
            }
        }
        assert_ne!(
            image(IconStyle::Outline, false).unwrap().rgba(),
            image(IconStyle::Solid, false).unwrap().rgba()
        );
    }

    #[test]
    fn existing_appearance_defaults_and_new_values_round_trip() {
        let previous = r#"{"theme":"dark","language":"en-US"}"#;
        let appearance: crate::settings::Appearance = serde_json::from_str(previous).unwrap();
        assert_eq!(appearance.icon_style, IconStyle::Outline);
        let selected: crate::settings::Appearance =
            serde_json::from_str(r#"{"iconStyle":"solid"}"#).unwrap();
        assert_eq!(selected.icon_style, IconStyle::Solid);
        assert_eq!(
            serde_json::to_value(selected).unwrap()["iconStyle"],
            "solid"
        );
        assert!(
            serde_json::from_str::<crate::settings::Appearance>(r#"{"iconStyle":"unknown"}"#)
                .is_err()
        );
    }
}
