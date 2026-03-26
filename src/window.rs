use anyhow::{Context, Result};
use log::debug;
use std::ffi::c_void;
use std::mem;
use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Graphics::Gdi::{MonitorFromWindow, MONITOR_DEFAULTTONEAREST};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindowLongW, GetWindowRect, GetWindowTextLengthW,
    GetWindowTextW, IsIconic, IsWindowVisible, GWL_EXSTYLE, WS_EX_APPWINDOW,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

use crate::monitor::{MonitorInfo, Rect};

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub rect: Rect,
    pub monitor_handle: isize,
}

impl WindowInfo {
    pub fn monitor_index(&self, monitors: &[MonitorInfo]) -> Option<u32> {
        monitors
            .iter()
            .find(|m| m.handle == self.monitor_handle)
            .map(|m| m.index)
    }
}

pub fn enumerate_windows(monitors: &[MonitorInfo]) -> Result<Vec<WindowInfo>> {
    let mut windows: Vec<WindowInfo> = Vec::new();

    unsafe {
        EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut windows as *mut Vec<WindowInfo> as isize),
        )
        .context("EnumWindows failed")?;
    }

    // Assign monitor handles
    for w in &mut windows {
        let hwnd = HWND(w.hwnd as *mut c_void);
        let hmon = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
        w.monitor_handle = hmon.0 as isize;
    }

    debug!("Found {} herdable windows", windows.len());
    for w in &windows {
        let mon_idx = w.monitor_index(monitors).unwrap_or(0);
        debug!("  [Display {}] {} ({})", mon_idx, w.title, w.rect);
    }

    Ok(windows)
}

pub fn windows_on_monitor<'a>(windows: &'a [WindowInfo], monitor: &MonitorInfo) -> Vec<&'a WindowInfo> {
    windows
        .iter()
        .filter(|w| w.monitor_handle == monitor.handle)
        .collect()
}

pub fn windows_not_on_monitor<'a>(windows: &'a [WindowInfo], monitor: &MonitorInfo) -> Vec<&'a WindowInfo> {
    windows
        .iter()
        .filter(|w| w.monitor_handle != monitor.handle)
        .collect()
}

/// Calculate cascade positions for windows on a target monitor.
/// Returns Vec of (x, y) positions. Window sizes are preserved.
pub fn cascade_positions(
    count: usize,
    work_area: &Rect,
    window_sizes: &[(i32, i32)],
) -> Vec<(i32, i32)> {
    const OFFSET: i32 = 30;
    let inset = OFFSET;

    let mut positions = Vec::with_capacity(count);
    let mut x = work_area.left + inset;
    let mut y = work_area.top + inset;
    let mut wrap_count: i32 = 0;

    for (i, &(w, h)) in window_sizes.iter().enumerate().take(count) {
        // Wrap if window would be >50% outside work area
        if x + w / 2 > work_area.right || y + h / 2 > work_area.bottom {
            wrap_count += 1;
            // Offset each wrap so windows don't stack exactly on top of each other
            x = work_area.left + inset + (wrap_count * OFFSET / 2);
            y = work_area.top + inset + (wrap_count * OFFSET / 2);
        }

        positions.push((x, y));

        if i < count - 1 {
            x += OFFSET;
            y += OFFSET;
        }
    }

    positions
}

pub fn move_window(hwnd: isize, x: i32, y: i32, w: i32, h: i32, restore_size: bool) -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER, SWP_NOSIZE,
    };

    let mut flags = SWP_NOZORDER | SWP_NOACTIVATE;
    if !restore_size {
        flags |= SWP_NOSIZE;
    }

    let handle = HWND(hwnd as *mut c_void);
    unsafe {
        SetWindowPos(handle, None, x, y, w, h, flags)
            .context("SetWindowPos failed")?;
    }
    Ok(())
}

/// Get the current title of a window by HWND, for identity validation.
pub fn get_window_title(hwnd: isize) -> Option<String> {
    let handle = HWND(hwnd as *mut c_void);
    unsafe {
        let len = GetWindowTextLengthW(handle);
        if len == 0 {
            return None;
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        let actual = GetWindowTextW(handle, &mut buf);
        if actual == 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..actual as usize]))
    }
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL { unsafe {
    let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);

    // Filter: must be visible
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }

    // Filter: must not be minimized
    if IsIconic(hwnd).as_bool() {
        return BOOL(1);
    }

    // Filter: check extended styles
    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;

    // Skip tool windows (unless they also have WS_EX_APPWINDOW)
    if (ex_style & WS_EX_TOOLWINDOW.0 != 0) && (ex_style & WS_EX_APPWINDOW.0 == 0) {
        return BOOL(1);
    }

    // Skip no-activate windows
    if ex_style & WS_EX_NOACTIVATE.0 != 0 {
        return BOOL(1);
    }

    // Filter: skip cloaked windows (suspended UWP apps)
    let mut cloaked: u32 = 0;
    if DwmGetWindowAttribute(
        hwnd,
        DWMWA_CLOAKED,
        &mut cloaked as *mut u32 as *mut c_void,
        mem::size_of::<u32>() as u32,
    )
    .is_ok()
        && cloaked != 0
    {
        return BOOL(1);
    }

    // Filter: must have a non-empty title
    let title_len = GetWindowTextLengthW(hwnd);
    if title_len == 0 {
        return BOOL(1);
    }

    let mut title_buf = vec![0u16; (title_len + 1) as usize];
    let actual_len = GetWindowTextW(hwnd, &mut title_buf);
    let title = String::from_utf16_lossy(&title_buf[..actual_len as usize]);

    // Filter: skip known shell/system window classes
    let mut class_buf = [0u16; 256];
    let class_len = GetClassNameW(hwnd, &mut class_buf);
    if class_len > 0 {
        let class_name = String::from_utf16_lossy(&class_buf[..class_len as usize]);
        const SKIP_CLASSES: &[&str] = &[
            "Progman",
            "WorkerW",
            "Shell_TrayWnd",
            "Shell_SecondaryTrayWnd",
            "Windows.UI.Core.CoreWindow",
            "XamlExplorerHostIslandWindow",
            "TopLevelWindowForOverflowXamlIsland",
        ];
        if SKIP_CLASSES.iter().any(|&c| c == class_name) {
            return BOOL(1);
        }
    }

    // Get window rect
    let mut rect = RECT::default();
    if GetWindowRect(hwnd, &mut rect).is_err() {
        return BOOL(1);
    }

    windows.push(WindowInfo {
        hwnd: hwnd.0 as isize,
        title,
        rect: Rect::from_win32(&rect),
        monitor_handle: 0, // assigned after enumeration
    });

    BOOL(1)
}}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitor::Rect;

    #[test]
    fn test_cascade_basic() {
        let work_area = Rect {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1040,
        };
        let sizes = vec![(800, 600), (800, 600), (800, 600)];
        let positions = cascade_positions(3, &work_area, &sizes);

        assert_eq!(positions.len(), 3);
        assert_eq!(positions[0], (30, 30));
        assert_eq!(positions[1], (60, 60));
        assert_eq!(positions[2], (90, 90));
    }

    #[test]
    fn test_cascade_wraps() {
        let work_area = Rect {
            left: 0,
            top: 0,
            right: 200,
            bottom: 200,
        };
        // Window is 150x150, after first at (30,30), next would be at (60,60)
        // 60 + 75 = 135 < 200, no wrap
        // next at (90, 90): 90 + 75 = 165 < 200, no wrap
        // next at (120, 120): 120 + 75 = 195 < 200, no wrap
        // next at (150, 150): 150 + 75 = 225 > 200, WRAP with offset
        let sizes = vec![(150, 150); 5];
        let positions = cascade_positions(5, &work_area, &sizes);

        assert_eq!(positions[0], (30, 30));
        // Wrapped: inset(30) + wrap_count(1) * OFFSET/2(15) = 45
        assert_eq!(positions[4], (45, 45));
    }

    #[test]
    fn test_cascade_empty() {
        let work_area = Rect {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };
        let positions = cascade_positions(0, &work_area, &[]);
        assert!(positions.is_empty());
    }
}
