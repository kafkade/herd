use std::io::{self, IsTerminal, Write};
use std::thread;
use std::time::Duration;

use crossterm::{
    cursor::{Hide, MoveUp, Show},
    style::{Color, Print, ResetColor, SetForegroundColor},
    execute, queue,
};

// Pixel map of the sheep logo (from logo.svg).
// Each char = one cell rendered as 2 block chars (██).
// ' ' = empty  'w' = wool  'd' = dark (head/ears/nose)  'f' = face (eyes)  'l' = leg
const SHEEP: [&str; 7] = [
    "   wwwww  ",
    "  wwwwwww ",
    "dddwwwwwww",
    " ffwwwwwww",
    " dwwwwwww ",
    "  wwwwwww ",
    "   ll l l ",
];

// FIGlet "herd" text
const HERD: [&str; 5] = [
    r"  _   _               _ ",
    r" | | | | ___ _ __ __| |",
    r" | |_| |/ _ \ '__/ _` |",
    r" |  _  |  __/ | | (_| |",
    r" |_| |_|\___|_|  \__,_|",
];

// Wool colors — green gradient matching the #6ee7b7 accent from logo.svg
const WOOL: [Color; 4] = [
    Color::Rgb {
        r: 110,
        g: 231,
        b: 183,
    },
    Color::Rgb {
        r: 94,
        g: 224,
        b: 171,
    },
    Color::Rgb {
        r: 78,
        g: 217,
        b: 159,
    },
    Color::Rgb {
        r: 62,
        g: 210,
        b: 147,
    },
];

const DARK: Color = Color::Rgb {
    r: 55,
    g: 57,
    b: 71,
};
const EYE: Color = Color::Rgb {
    r: 240,
    g: 235,
    b: 229,
};
const HIGHLIGHT: Color = Color::Rgb {
    r: 255,
    g: 255,
    b: 255,
};

const TEXT_GRAD: [Color; 5] = [
    Color::Rgb {
        r: 76,
        g: 217,
        b: 100,
    },
    Color::Rgb {
        r: 64,
        g: 224,
        b: 130,
    },
    Color::Rgb {
        r: 52,
        g: 231,
        b: 160,
    },
    Color::Rgb {
        r: 40,
        g: 238,
        b: 190,
    },
    Color::Rgb {
        r: 30,
        g: 245,
        b: 220,
    },
];

#[derive(Clone, Copy)]
struct Pixel {
    ch: char,
    color: Color,
}

fn wool_color(row: usize, col: usize) -> Color {
    WOOL[(row + col) % WOOL.len()]
}

fn blend(a: Color, b: Color, t: f32) -> Color {
    if let (
        Color::Rgb {
            r: r1,
            g: g1,
            b: b1,
        },
        Color::Rgb {
            r: r2,
            g: g2,
            b: b2,
        },
    ) = (a, b)
    {
        Color::Rgb {
            r: (r1 as f32 + (r2 as f32 - r1 as f32) * t) as u8,
            g: (g1 as f32 + (g2 as f32 - g1 as f32) * t) as u8,
            b: (b1 as f32 + (b2 as f32 - b1 as f32) * t) as u8,
        }
    } else {
        a
    }
}

/// Build the full banner as a 2D grid of pixels (character + color).
fn build_banner() -> Vec<Vec<Pixel>> {
    let mut rows = Vec::new();
    let pad = 3;

    for (r, line) in SHEEP.iter().enumerate() {
        let mut row: Vec<Pixel> = (0..pad)
            .map(|_| Pixel {
                ch: ' ',
                color: Color::Reset,
            })
            .collect();

        for (c, cell) in line.chars().enumerate() {
            match cell {
                'w' => {
                    let clr = wool_color(r, c);
                    row.push(Pixel { ch: '█', color: clr });
                    row.push(Pixel { ch: '█', color: clr });
                }
                'd' | 'l' => {
                    row.push(Pixel {
                        ch: '█',
                        color: DARK,
                    });
                    row.push(Pixel {
                        ch: '█',
                        color: DARK,
                    });
                }
                'f' => {
                    // Eyes: bright pixel on each face cell, dark on the other side
                    // Gives  █·  ·█  → separated eye pair within the face
                    if c == 1 {
                        row.push(Pixel {
                            ch: '█',
                            color: DARK,
                        });
                        row.push(Pixel {
                            ch: '█',
                            color: EYE,
                        });
                    } else {
                        row.push(Pixel {
                            ch: '█',
                            color: EYE,
                        });
                        row.push(Pixel {
                            ch: '█',
                            color: DARK,
                        });
                    }
                }
                _ => {
                    row.push(Pixel {
                        ch: ' ',
                        color: Color::Reset,
                    });
                    row.push(Pixel {
                        ch: ' ',
                        color: Color::Reset,
                    });
                }
            }
        }
        rows.push(row);
    }

    // Blank separator
    rows.push(vec![]);

    // "herd" text
    for (i, line) in HERD.iter().enumerate() {
        let clr = TEXT_GRAD[i];
        rows.push(line.chars().map(|ch| Pixel { ch, color: clr }).collect());
    }

    rows
}

fn draw_banner(out: &mut impl Write, banner: &[Vec<Pixel>], beam: Option<(i32, i32)>) {
    for row in banner {
        for (j, px) in row.iter().enumerate() {
            if px.ch == ' ' {
                queue!(out, Print(' ')).ok();
            } else if let Some((pos, radius)) = beam {
                let dist = (pos - j as i32).abs();
                let color = if dist == 0 {
                    HIGHLIGHT
                } else if dist <= radius {
                    blend(px.color, HIGHLIGHT, 1.0 - dist as f32 / radius as f32)
                } else {
                    px.color
                };
                queue!(out, SetForegroundColor(color), Print(px.ch)).ok();
            } else {
                queue!(out, SetForegroundColor(px.color), Print(px.ch)).ok();
            }
        }
        queue!(out, Print("\n")).ok();
    }
}

pub fn print_version() {
    let version = env!("CARGO_PKG_VERSION");

    if !io::stdout().is_terminal() {
        println!("herd {version}");
        return;
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let banner = build_banner();
    let total_rows = banner.len() as u16;

    execute!(out, Hide).ok();

    // Phase 1 — reveal row by row with per-character delay
    for row in &banner {
        for px in row {
            if px.ch == ' ' {
                queue!(out, Print(' ')).ok();
            } else {
                queue!(out, SetForegroundColor(px.color), Print(px.ch)).ok();
            }
            out.flush().ok();
            thread::sleep(Duration::from_millis(2));
        }
        queue!(out, Print("\n")).ok();
        out.flush().ok();
        thread::sleep(Duration::from_millis(20));
    }

    thread::sleep(Duration::from_millis(60));

    // Phase 2 — shimmer sweep (bright highlight beam left → right)
    let max_width = banner.iter().map(|r| r.len()).max().unwrap_or(0) as i32;
    let beam_radius = 4i32;

    for pos in -beam_radius..max_width + beam_radius {
        execute!(out, MoveUp(total_rows)).ok();
        draw_banner(&mut out, &banner, Some((pos, beam_radius)));
        out.flush().ok();
        thread::sleep(Duration::from_millis(8));
    }

    // Phase 3 — final clean render
    execute!(out, MoveUp(total_rows)).ok();
    draw_banner(&mut out, &banner, None);
    out.flush().ok();
    execute!(out, ResetColor).ok();

    // Phase 4 — version info
    thread::sleep(Duration::from_millis(80));
    queue!(out, Print("\n")).ok();

    queue!(
        out,
        SetForegroundColor(Color::Rgb {
            r: 160,
            g: 160,
            b: 160,
        }),
        Print("  \u{1f411} herd "),
        SetForegroundColor(TEXT_GRAD[2]),
        Print(format!("v{version}")),
        ResetColor,
        Print("\n"),
    )
    .ok();

    queue!(
        out,
        SetForegroundColor(Color::Rgb {
            r: 120,
            g: 120,
            b: 120,
        }),
        Print("  Move all windows to one display\n"),
        ResetColor,
        Print("\n"),
    )
    .ok();

    execute!(out, Show).ok();
}
