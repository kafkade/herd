use anyhow::Result;
use log::debug;
use std::mem;
use windows::core::BOOL;
use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
};

#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub handle: isize,
    pub index: u32,
    pub device_name: String,
    pub rect: Rect,
    pub work_area: Rect,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    pub fn from_win32(r: &RECT) -> Self {
        Rect {
            left: r.left,
            top: r.top,
            right: r.right,
            bottom: r.bottom,
        }
    }
}

impl std::fmt::Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{} @ ({},{})", self.width(), self.height(), self.left, self.top)
    }
}

pub fn enumerate_monitors() -> Result<Vec<MonitorInfo>> {
    let mut monitors: Vec<MonitorInfo> = Vec::new();

    unsafe {
        let result = EnumDisplayMonitors(
            None,
            None,
            Some(enum_monitors_callback),
            LPARAM(&mut monitors as *mut Vec<MonitorInfo> as isize),
        );
        if !result.as_bool() {
            anyhow::bail!("EnumDisplayMonitors failed");
        }
    }

    // Sort: primary first, then by position (left-to-right, top-to-bottom)
    monitors.sort_by(|a, b| {
        b.is_primary
            .cmp(&a.is_primary)
            .then(a.rect.left.cmp(&b.rect.left))
            .then(a.rect.top.cmp(&b.rect.top))
    });

    // Assign 1-based indices after sorting
    for (i, m) in monitors.iter_mut().enumerate() {
        m.index = (i + 1) as u32;
    }

    debug!("Found {} monitors", monitors.len());
    for m in &monitors {
        debug!(
            "  Display {}: {} {} {}{}",
            m.index,
            m.device_name,
            m.rect,
            m.work_area,
            if m.is_primary { " (Primary)" } else { "" }
        );
    }

    Ok(monitors)
}

pub fn find_target_monitor(monitors: &[MonitorInfo], display: Option<u32>) -> Option<&MonitorInfo> {
    match display {
        Some(n) => monitors.iter().find(|m| m.index == n),
        None => monitors.iter().find(|m| m.is_primary),
    }
}

unsafe extern "system" fn enum_monitors_callback(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _clip: *mut RECT,
    lparam: LPARAM,
) -> BOOL { unsafe {
    let monitors = &mut *(lparam.0 as *mut Vec<MonitorInfo>);

    let mut info: MONITORINFOEXW = mem::zeroed();
    info.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;

    if GetMonitorInfoW(hmonitor, &mut info as *mut MONITORINFOEXW as *mut _).as_bool() {
        let device_name_len = info
            .szDevice
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(info.szDevice.len());
        let device_name = String::from_utf16_lossy(&info.szDevice[..device_name_len]);

        let is_primary = (info.monitorInfo.dwFlags & 1) != 0; // MONITORINFOF_PRIMARY

        monitors.push(MonitorInfo {
            handle: hmonitor.0 as isize,
            index: 0, // assigned after sorting
            device_name,
            rect: Rect::from_win32(&info.monitorInfo.rcMonitor),
            work_area: Rect::from_win32(&info.monitorInfo.rcWork),
            is_primary,
        });
    }

    BOOL(1)
}}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_dimensions() {
        let r = Rect {
            left: 100,
            top: 50,
            right: 1380,
            bottom: 818,
        };
        assert_eq!(r.width(), 1280);
        assert_eq!(r.height(), 768);
    }

    #[test]
    fn test_rect_display() {
        let r = Rect {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };
        assert_eq!(format!("{}", r), "1920x1080 @ (0,0)");
    }

    #[test]
    fn test_find_target_primary() {
        let monitors = vec![
            MonitorInfo {
                handle: 1,
                index: 1,
                device_name: "\\\\.\\DISPLAY1".to_string(),
                rect: Rect { left: 0, top: 0, right: 1920, bottom: 1080 },
                work_area: Rect { left: 0, top: 0, right: 1920, bottom: 1040 },
                is_primary: true,
            },
            MonitorInfo {
                handle: 2,
                index: 2,
                device_name: "\\\\.\\DISPLAY2".to_string(),
                rect: Rect { left: 1920, top: 0, right: 3840, bottom: 1080 },
                work_area: Rect { left: 1920, top: 0, right: 3840, bottom: 1040 },
                is_primary: false,
            },
        ];
        let target = find_target_monitor(&monitors, None).unwrap();
        assert!(target.is_primary);
        assert_eq!(target.index, 1);
    }

    #[test]
    fn test_find_target_by_index() {
        let monitors = vec![
            MonitorInfo {
                handle: 1,
                index: 1,
                device_name: "\\\\.\\DISPLAY1".to_string(),
                rect: Rect { left: 0, top: 0, right: 1920, bottom: 1080 },
                work_area: Rect { left: 0, top: 0, right: 1920, bottom: 1040 },
                is_primary: true,
            },
            MonitorInfo {
                handle: 2,
                index: 2,
                device_name: "\\\\.\\DISPLAY2".to_string(),
                rect: Rect { left: 1920, top: 0, right: 3840, bottom: 1080 },
                work_area: Rect { left: 1920, top: 0, right: 3840, bottom: 1040 },
                is_primary: false,
            },
        ];
        let target = find_target_monitor(&monitors, Some(2)).unwrap();
        assert_eq!(target.index, 2);
        assert!(!target.is_primary);
    }
}
