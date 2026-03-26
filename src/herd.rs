use anyhow::Result;
use log::{info, warn};

use crate::error::HerdError;
use crate::monitor;
use crate::snapshot;
use crate::window;

pub fn run(display: Option<u32>, dry_run: bool) -> Result<()> {
    let monitors = monitor::enumerate_monitors()?;

    if monitors.len() < 2 {
        println!("Only one display detected. Nothing to herd.");
        return Ok(());
    }

    let target = monitor::find_target_monitor(&monitors, display).ok_or_else(|| {
        HerdError::InvalidDisplay(display.unwrap_or(0), monitors.len() as u32)
    })?;

    println!(
        "🐑 Herding windows to Display {} ({}{})...",
        target.index,
        target.rect,
        if target.is_primary { ", Primary" } else { "" }
    );

    let windows = window::enumerate_windows(&monitors)?;
    let to_move = window::windows_not_on_monitor(&windows, target);

    if to_move.is_empty() {
        println!("All windows are already on Display {}. Nothing to move.", target.index);
        return Ok(());
    }

    // Save snapshot of windows we're about to move
    let monitor_indices: Vec<u32> = to_move
        .iter()
        .map(|w| w.monitor_index(&monitors).unwrap_or(0))
        .collect();

    let original_windows: Vec<_> = to_move
        .iter()
        .map(|w| (*w).clone())
        .collect();

    if !dry_run {
        snapshot::save_snapshot(
            &original_windows,
            &monitor_indices,
            target.index,
        )?;
    }

    // Calculate cascade positions
    let sizes: Vec<(i32, i32)> = to_move
        .iter()
        .map(|w| (w.rect.width(), w.rect.height()))
        .collect();
    let positions = window::cascade_positions(to_move.len(), &target.work_area, &sizes);

    let mut moved = 0;
    let mut failed = 0;

    for (w, &(x, y)) in to_move.iter().zip(positions.iter()) {
        let from_display = w.monitor_index(&monitors).unwrap_or(0);

        if dry_run {
            println!(
                "  Would move \"{}\" from Display {} to Display {} at ({}, {})",
                w.title, from_display, target.index, x, y
            );
        } else {
            match window::move_window(w.hwnd, x, y, w.rect.width(), w.rect.height(), false) {
                Ok(()) => {
                    info!("Moved \"{}\" from Display {} to ({}, {})", w.title, from_display, x, y);
                    moved += 1;
                }
                Err(e) => {
                    warn!(
                        "Could not move \"{}\": {}. Try running as administrator.",
                        w.title, e
                    );
                    failed += 1;
                }
            }
        }
    }

    if dry_run {
        println!("\nDry run: {} window(s) would be moved.", to_move.len());
    } else {
        println!("✅ Herded {} window(s) to Display {}.", moved, target.index);
        if failed > 0 {
            println!(
                "⚠️  {} window(s) could not be moved (try running as administrator).",
                failed
            );
        }
    }

    Ok(())
}

pub fn list_displays() -> Result<()> {
    let monitors = monitor::enumerate_monitors()?;
    let windows = window::enumerate_windows(&monitors)?;

    println!("Displays:");
    println!("{:<10} {:<10} {:<20} {:<20} Windows", "Display", "Primary", "Resolution", "Position");
    println!("{}", "-".repeat(75));

    for m in &monitors {
        let win_count = window::windows_on_monitor(&windows, m).len();
        println!(
            "{:<10} {:<10} {:<20} {:<20} {}",
            m.index,
            if m.is_primary { "✓" } else { "" },
            format!("{}x{}", m.rect.width(), m.rect.height()),
            format!("({}, {})", m.rect.left, m.rect.top),
            win_count,
        );
    }

    println!("\nTotal: {} display(s), {} window(s)", monitors.len(), windows.len());

    Ok(())
}

pub fn undo() -> Result<()> {
    let result = snapshot::load_latest_snapshot()?;

    match result {
        None => {
            println!("No previous herd operation to undo.");
            return Ok(());
        }
        Some((snap, snapshot_path)) => {
            println!("🔄 Restoring {} window(s) from {}...", snap.windows.len(), snap.timestamp);

            let mut restored = 0;
            let mut skipped = 0;
            let mut failed = 0;

            for ws in &snap.windows {
                // Validate HWND identity: check current window title matches snapshot
                match window::get_window_title(ws.hwnd) {
                    None => {
                        warn!("Skipping \"{}\": window no longer exists", ws.title);
                        skipped += 1;
                        continue;
                    }
                    Some(current_title) if current_title != ws.title => {
                        warn!(
                            "Skipping \"{}\": HWND reused by different window \"{}\"",
                            ws.title, current_title
                        );
                        skipped += 1;
                        continue;
                    }
                    _ => {}
                }

                match window::move_window(
                    ws.hwnd,
                    ws.rect.left,
                    ws.rect.top,
                    ws.rect.width(),
                    ws.rect.height(),
                    true, // restore original size (important for mixed-DPI)
                ) {
                    Ok(()) => {
                        info!(
                            "Restored \"{}\" to ({}, {})",
                            ws.title, ws.rect.left, ws.rect.top
                        );
                        restored += 1;
                    }
                    Err(e) => {
                        warn!(
                            "Could not restore \"{}\": {} (window may have been closed)",
                            ws.title, e
                        );
                        failed += 1;
                    }
                }
            }

            println!("✅ Restored {} window(s).", restored);
            if skipped > 0 {
                println!(
                    "⏭️  {} window(s) skipped (closed or HWND reused).",
                    skipped
                );
            }
            if failed > 0 {
                println!(
                    "⚠️  {} window(s) could not be restored (may have been closed).",
                    failed
                );
            }

            // Only remove snapshot after successful restore attempt
            if let Err(e) = snapshot::remove_snapshot(&snapshot_path) {
                warn!("Could not clean up snapshot: {}", e);
            }
        }
    }

    Ok(())
}
