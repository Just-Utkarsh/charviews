use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

/// Dark → light (invert for “negative” look by swapping ends in render).
const RAMP: &[u8] = br#" .'`^",:;Il!i><~+_-?][}{1)(|\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$"#;

fn parse_device(s: Option<&str>) -> CameraIndex {
    match s {
        None => CameraIndex::Index(0),
        Some(st) => {
            let st = st.trim();
            if let Ok(n) = st.parse::<u32>() {
                CameraIndex::Index(n)
            } else {
                CameraIndex::String(st.to_string())
            }
        }
    }
}

pub fn run(device: Option<&str>, fps: u32) -> Result<()> {
    let index = parse_device(device);
    let fps = fps.clamp(1, 120);
    let frame_budget = Duration::from_secs_f64(1.0 / fps as f64);

    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestFrameRate(fps));
    let mut camera = Camera::new(index.clone(), requested)
        .or_else(|_| {
            Camera::new(
                index,
                RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
            )
        })
        .context("open camera")?;
    camera.open_stream().context("start camera stream")?;

    let mut stdout = stdout();
    enable_raw_mode().context("enable terminal raw mode")?;
    execute!(stdout, Hide)?;

    let mut rgb: Vec<u8> = Vec::new();

    let mut out_cols: u16 = 0;
    let mut out_rows: u16 = 0;
    let mut line_buf: Vec<u8> = Vec::new();
    let mut quit = false;
    let mut next_tick = Instant::now();

    let result = (|| -> Result<()> {
        loop {
            if event::poll(Duration::ZERO)? {
                match event::read()? {
                    Event::Key(key) => {
                        if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                            quit = true;
                        }
                    }
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
            if quit {
                break;
            }

            let (tw, th) = size().unwrap_or(if out_cols > 0 && out_rows > 0 {
                (out_cols, out_rows)
            } else {
                (80u16, 24u16)
            });
            let cols = tw.max(1);
            let rows = th.max(1);

            if cols != out_cols || rows != out_rows {
                out_cols = cols;
                out_rows = rows;
                let line_len = cols as usize;
                line_buf.clear();
                line_buf.reserve(line_len);
            }

            let buf = camera.frame().context("capture frame")?;
            let cam_w = buf.resolution().width_x as usize;
            let cam_h = buf.resolution().height_y as usize;
            let need = cam_w * cam_h * 3;
            if rgb.len() < need {
                rgb.resize(need, 0);
            }
            buf.decode_image_to_buffer::<RgbFormat>(&mut rgb[..need])
                .context("decode frame to RGB")?;
            if cam_w == 0 || cam_h == 0 {
                continue;
            }

            let gw = cols as usize;
            let gh = rows as usize;

            execute!(stdout, MoveTo(0, 0), Clear(ClearType::FromCursorDown))?;

            for cy in 0..gh {
                line_buf.clear();
                for cx in 0..gw {
                    let sx = ((cx as u64 * cam_w as u64 * 2 + cam_w as u64) / (gw as u64 * 2)) as usize;
                    let sy = ((cy as u64 * cam_h as u64 * 2 + cam_h as u64) / (gh as u64 * 2)) as usize;
                    let sx = sx.min(cam_w - 1);
                    let sy = sy.min(cam_h - 1);
                    let i = (sy * cam_w + sx) * 3;
                    let r = rgb[i] as u32;
                    let g = rgb[i + 1] as u32;
                    let b = rgb[i + 2] as u32;
                    let y = (r * 77 + g * 150 + b * 29) >> 8;
                    let idx = (y as usize * (RAMP.len() - 1)) / 255;
                    line_buf.push(RAMP[idx]);
                }
                stdout.write_all(&line_buf)?;
                if cy + 1 < gh {
                    stdout.write_all(b"\r\n")?;
                }
            }
            stdout.flush()?;

            let now = Instant::now();
            if now < next_tick {
                let remain = next_tick - now;
                let slice = remain.min(Duration::from_millis(50));
                if event::poll(slice)? {
                    match event::read()? {
                        Event::Key(key) => {
                            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                                quit = true;
                            }
                        }
                        Event::Resize(_, _) => {}
                        _ => {}
                    }
                }
                let now2 = Instant::now();
                if now2 < next_tick {
                    std::thread::sleep(next_tick - now2);
                }
            }
            next_tick += frame_budget;
            if next_tick < Instant::now() {
                next_tick = Instant::now();
            }
        }
        Ok(())
    })();

    let _ = execute!(stdout, Show);
    let _ = disable_raw_mode();
    let _ = stdout.flush();

    camera.stop_stream().ok();
    result
}
