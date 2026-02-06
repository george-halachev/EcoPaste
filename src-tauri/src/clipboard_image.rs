#[cfg(target_os = "windows")]
mod win {
    use serde::Serialize;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::io::Cursor;
    use std::path::PathBuf;
    use tauri::{AppHandle, Manager, Runtime};

    const CF_DIB: u32 = 8;
    const CF_DIBV5: u32 = 17;

    #[repr(C)]
    #[allow(non_snake_case)]
    struct BITMAPINFOHEADER {
        biSize: u32,
        biWidth: i32,
        biHeight: i32,
        biPlanes: u16,
        biBitCount: u16,
        biCompression: u32,
        biSizeImage: u32,
        biXPelsPerMeter: i32,
        biYPelsPerMeter: i32,
        biClrUsed: u32,
        biClrImportant: u32,
    }

    extern "system" {
        fn OpenClipboard(hWndNewOwner: *mut std::ffi::c_void) -> i32;
        fn CloseClipboard() -> i32;
        fn GetClipboardData(uFormat: u32) -> *mut std::ffi::c_void;
        fn IsClipboardFormatAvailable(format: u32) -> i32;
        fn RegisterClipboardFormatW(lpszFormat: *const u16) -> u32;
        fn GlobalLock(hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        fn GlobalUnlock(hMem: *mut std::ffi::c_void) -> i32;
        fn GlobalSize(hMem: *mut std::ffi::c_void) -> usize;
    }

    fn register_png_format() -> u32 {
        let name: Vec<u16> = "PNG\0".encode_utf16().collect();
        unsafe { RegisterClipboardFormatW(name.as_ptr()) }
    }

    fn get_images_dir<R: Runtime>(app_handle: &AppHandle<R>) -> PathBuf {
        let app_data = app_handle.path().app_data_dir().unwrap();
        app_data.join("tauri-plugin-clipboard-x").join("images")
    }

    fn hash_bytes(data: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    fn get_png_dimensions(data: &[u8]) -> Option<(u32, u32)> {
        if data.len() < 24 {
            return None;
        }
        if &data[0..8] != &[137, 80, 78, 71, 13, 10, 26, 10] {
            return None;
        }
        let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        Some((width, height))
    }

    #[derive(Serialize, Clone)]
    pub struct ReadImageResult {
        pub path: String,
        pub size: u64,
        pub width: u32,
        pub height: u32,
    }

    /// Check if the Windows clipboard has any image format available.
    pub fn has_image() -> bool {
        unsafe {
            let png_format = register_png_format();
            IsClipboardFormatAvailable(png_format) != 0
                || IsClipboardFormatAvailable(CF_DIBV5) != 0
                || IsClipboardFormatAvailable(CF_DIB) != 0
        }
    }

    /// Read an image from the Windows clipboard and save it as PNG.
    /// Returns None if no supported image format is found.
    pub fn read_image<R: Runtime>(
        app_handle: &AppHandle<R>,
    ) -> Result<Option<ReadImageResult>, String> {
        unsafe {
            if OpenClipboard(std::ptr::null_mut()) == 0 {
                return Err("Failed to open clipboard".to_string());
            }

            let result = read_image_inner(app_handle);
            CloseClipboard();
            result
        }
    }

    unsafe fn read_image_inner<R: Runtime>(
        app_handle: &AppHandle<R>,
    ) -> Result<Option<ReadImageResult>, String> {
        // Try registered PNG format first (best quality, no conversion needed)
        let png_format = register_png_format();
        if IsClipboardFormatAvailable(png_format) != 0 {
            if let Some(result) = try_read_raw_format(app_handle, png_format, true)? {
                return Ok(Some(result));
            }
        }

        // Try CF_DIBV5 (DIB with extended color info)
        if IsClipboardFormatAvailable(CF_DIBV5) != 0 {
            if let Some(result) = try_read_dib(app_handle, CF_DIBV5)? {
                return Ok(Some(result));
            }
        }

        // Try CF_DIB (standard device-independent bitmap)
        if IsClipboardFormatAvailable(CF_DIB) != 0 {
            if let Some(result) = try_read_dib(app_handle, CF_DIB)? {
                return Ok(Some(result));
            }
        }

        Ok(None)
    }

    /// Read a raw clipboard format that is already PNG data.
    unsafe fn try_read_raw_format<R: Runtime>(
        app_handle: &AppHandle<R>,
        format: u32,
        is_png: bool,
    ) -> Result<Option<ReadImageResult>, String> {
        let handle = GetClipboardData(format);
        if handle.is_null() {
            return Ok(None);
        }

        let data = GlobalLock(handle);
        if data.is_null() {
            return Ok(None);
        }

        let size = GlobalSize(handle);
        if size == 0 {
            GlobalUnlock(handle);
            return Ok(None);
        }

        let bytes = std::slice::from_raw_parts(data as *const u8, size);
        let result = if is_png {
            save_png_bytes(app_handle, bytes)
        } else {
            Err("Unsupported raw format".to_string())
        };

        GlobalUnlock(handle);
        result
    }

    /// Read a DIB/DIBV5 format, convert to PNG, and save.
    unsafe fn try_read_dib<R: Runtime>(
        app_handle: &AppHandle<R>,
        format: u32,
    ) -> Result<Option<ReadImageResult>, String> {
        let handle = GetClipboardData(format);
        if handle.is_null() {
            return Ok(None);
        }

        let data = GlobalLock(handle);
        if data.is_null() {
            return Ok(None);
        }

        let size = GlobalSize(handle);
        if size < std::mem::size_of::<BITMAPINFOHEADER>() {
            GlobalUnlock(handle);
            return Ok(None);
        }

        let bytes = std::slice::from_raw_parts(data as *const u8, size);
        let header = &*(data as *const BITMAPINFOHEADER);

        let bit_count = header.biBitCount as u32;

        // Calculate color table size
        let colors_used = if header.biClrUsed > 0 {
            header.biClrUsed
        } else if bit_count <= 8 {
            1u32 << bit_count
        } else {
            0
        };
        let color_table_size = colors_used * 4; // RGBQUAD = 4 bytes each

        let header_size = header.biSize;
        let pixel_data_offset = header_size + color_table_size;
        if size as u32 <= pixel_data_offset {
            GlobalUnlock(handle);
            return Ok(None);
        }

        // Build a complete BMP file: file header + DIB data
        let bmp_file_header_size: u32 = 14;
        let file_size = bmp_file_header_size + size as u32;
        let pixel_offset = bmp_file_header_size + pixel_data_offset;

        let mut bmp_data = Vec::with_capacity(file_size as usize);
        // BMP file header
        bmp_data.extend_from_slice(&[0x42, 0x4D]); // 'BM'
        bmp_data.extend_from_slice(&file_size.to_le_bytes());
        bmp_data.extend_from_slice(&[0, 0, 0, 0]); // reserved
        bmp_data.extend_from_slice(&pixel_offset.to_le_bytes());
        // DIB data (header + color table + pixels)
        bmp_data.extend_from_slice(bytes);

        GlobalUnlock(handle);

        // Convert BMP to PNG using the image crate
        convert_bmp_to_png(app_handle, &bmp_data)
    }

    fn save_png_bytes<R: Runtime>(
        app_handle: &AppHandle<R>,
        png_bytes: &[u8],
    ) -> Result<Option<ReadImageResult>, String> {
        if png_bytes.is_empty() {
            return Ok(None);
        }

        let (width, height) = get_png_dimensions(png_bytes).unwrap_or((0, 0));
        if width == 0 || height == 0 {
            return Ok(None);
        }

        let hash = hash_bytes(png_bytes);
        let filename = format!("{}.png", hash);
        let dir = get_images_dir(app_handle);
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(&filename);

        // Only write if not already present (hash-based dedup on disk)
        if !path.exists() {
            std::fs::write(&path, png_bytes).map_err(|e| e.to_string())?;
        }

        let file_size = std::fs::metadata(&path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(Some(ReadImageResult {
            path: path.to_string_lossy().to_string(),
            size: file_size,
            width,
            height,
        }))
    }

    fn convert_bmp_to_png<R: Runtime>(
        app_handle: &AppHandle<R>,
        bmp_data: &[u8],
    ) -> Result<Option<ReadImageResult>, String> {
        let img = image::load_from_memory_with_format(bmp_data, image::ImageFormat::Bmp)
            .map_err(|e| format!("Failed to decode BMP: {}", e))?;

        let width = img.width();
        let height = img.height();

        if width == 0 || height == 0 {
            return Ok(None);
        }

        let mut png_bytes = Vec::new();
        img.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;

        save_png_bytes(app_handle, &png_bytes)
    }

    #[tauri::command]
    pub async fn has_clipboard_image_win() -> bool {
        has_image()
    }

    #[tauri::command]
    pub async fn read_clipboard_image_win<R: Runtime>(
        app_handle: AppHandle<R>,
    ) -> Result<Option<ReadImageResult>, String> {
        read_image(&app_handle)
    }
}

#[cfg(target_os = "windows")]
pub use win::{has_clipboard_image_win, read_clipboard_image_win};

// Stubs for non-Windows platforms
#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub async fn has_clipboard_image_win() -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub async fn read_clipboard_image_win() -> Result<Option<()>, String> {
    Ok(None)
}
