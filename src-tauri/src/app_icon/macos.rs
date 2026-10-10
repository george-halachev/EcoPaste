use anyhow::Context;
use objc2::{AnyThread, MainThreadMarker};
use objc2_app_kit::{NSApplication, NSImage};
use objc2_foundation::NSData;

use crate::core::Result;

/// NSApplication's runtime icon controls the Dock independently of bundle assets.
pub fn set_dock_icon(png: &[u8]) -> Result<()> {
    let marker = MainThreadMarker::new().context("Dock icon requires the main thread")?;
    let data = NSData::with_bytes(png);
    let icon = NSImage::initWithData(NSImage::alloc(), &data).context("decode Dock icon")?;
    let application = NSApplication::sharedApplication(marker);
    unsafe {
        application.setApplicationIconImage(Some(&icon));
    }
    Ok(())
}
