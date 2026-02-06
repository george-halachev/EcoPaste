use super::wait;
use std::ffi::OsString;
use std::mem;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use std::sync::Mutex;
use tauri::command;
use tauri_plugin_eco_window::MAIN_WINDOW_TITLE;
use winapi::shared::minwindef::DWORD;
use winapi::shared::windef::{HWINEVENTHOOK, HWND};
use winapi::um::winuser::{
    GetWindowTextLengthW, GetWindowTextW, SendInput, SetForegroundWindow, SetWinEventHook,
    INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT,
    VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT, VK_INSERT,
};

static PREVIOUS_WINDOW: Mutex<Option<isize>> = Mutex::new(None);

// 获取窗口标题
unsafe fn get_window_title(hwnd: HWND) -> String {
    let length = GetWindowTextLengthW(hwnd);

    if length == 0 {
        return String::new();
    }

    let mut buffer: Vec<u16> = vec![0; (length + 1) as usize];

    GetWindowTextW(hwnd, buffer.as_mut_ptr(), length + 1);

    OsString::from_wide(&buffer[..length as usize])
        .to_string_lossy()
        .into_owned()
}

// 定义事件钩子回调函数
unsafe extern "system" fn event_hook_callback(
    _h_win_event_hook: HWINEVENTHOOK,
    event: DWORD,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _dw_event_thread: DWORD,
    _dwms_event_time: DWORD,
) {
    if event == EVENT_SYSTEM_FOREGROUND {
        let window_title = get_window_title(hwnd);

        if window_title == MAIN_WINDOW_TITLE {
            return;
        }

        let mut previous_window = PREVIOUS_WINDOW.lock().unwrap();
        let _ = previous_window.insert(hwnd as isize);
    }
}

// 监听窗口切换
pub fn observe_app() {
    unsafe {
        // 设置事件钩子
        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            ptr::null_mut(),
            Some(event_hook_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        if hook.is_null() {
            log::error!("设置事件钩子失败");
            return;
        }
    }
}

// 获取上一个窗口
pub fn get_previous_window() -> Option<isize> {
    return PREVIOUS_WINDOW.lock().unwrap().clone();
}

// 聚焦上一个窗口
fn focus_previous_window() {
    unsafe {
        let hwnd = match get_previous_window() {
            Some(hwnd) => hwnd as HWND,
            None => return,
        };

        if hwnd.is_null() {
            return;
        }

        SetForegroundWindow(hwnd);
    }
}

fn make_key_input(vk: u16, flags: u32) -> INPUT {
    let mut input: INPUT = unsafe { mem::zeroed() };
    input.type_ = INPUT_KEYBOARD;
    unsafe {
        let ki = input.u.ki_mut();
        *ki = KEYBDINPUT {
            wVk: vk,
            wScan: 0,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: 0,
        };
    }
    input
}

// 粘贴
#[command]
pub async fn paste() {
    focus_previous_window();

    wait(100);

    // Release any modifier keys that may still be physically held down,
    // then send Shift+Insert for paste, all as one atomic SendInput call.
    let inputs = vec![
        // Release modifiers that could interfere
        make_key_input(VK_CONTROL as u16, KEYEVENTF_KEYUP),
        make_key_input(VK_SHIFT as u16, KEYEVENTF_KEYUP),
        make_key_input(VK_MENU as u16, KEYEVENTF_KEYUP),
        make_key_input(VK_LWIN as u16, KEYEVENTF_KEYUP),
        make_key_input(VK_RWIN as u16, KEYEVENTF_KEYUP),
        // Shift+Insert (paste)
        make_key_input(VK_SHIFT as u16, 0),
        make_key_input(VK_INSERT as u16, 0),
        make_key_input(VK_INSERT as u16, KEYEVENTF_KEYUP),
        make_key_input(VK_SHIFT as u16, KEYEVENTF_KEYUP),
    ];

    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr() as *mut INPUT,
            mem::size_of::<INPUT>() as i32,
        );
    }
}
