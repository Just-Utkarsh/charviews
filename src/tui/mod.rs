mod logo;

use crate::camera_list;
use crate::terminal_spawn::{spawn_viewer, DEFAULT_VIEWER_GEOMETRY};
use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle},
};
use logo::{LOGO_LINES, LOGO_TAGLINE};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    DefaultTerminal, Frame,
};
use std::env;
use std::io::stdout;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Panel {
    Cameras,
    Fps,
}

pub fn run() -> Result<()> {
    let mut stdout = stdout();
    enable_raw_mode().context("raw mode")?;
    execute!(
        stdout,
        EnterAlternateScreen,
        SetTitle("Charvision — dashboard")
    )?;
    let mut terminal = ratatui::init();
    let app_result = run_inner(&mut terminal);
    ratatui::restore();
    let _ = execute!(stdout, SetTitle(""), LeaveAlternateScreen);
    disable_raw_mode()?;
    app_result
}

fn run_inner(terminal: &mut DefaultTerminal) -> Result<()> {
    const FPS_CHOICES: [u32; 4] = [15, 24, 30, 60];

    let mut state = AppState {
        cameras: Vec::new(),
        camera_list: ListState::default().with_selected(Some(0)),
        fps_list: ListState::default().with_selected(Some(2)),
        focus: Panel::Cameras,
        status: String::new(),
        fps_choices: FPS_CHOICES,
    };

    match camera_list::list_cameras() {
        Ok(c) if !c.is_empty() => {
            state.cameras = c;
        }
        Ok(_) => {
            state.status =
                "No cameras found. Check video devices and permissions (e.g. video group).".into();
        }
        Err(e) => {
            state.status = format!("{e:#}");
        }
    }

    loop {
        terminal.draw(|f| ui(f, &mut state))?;

        let evt = event::read()?;
        if let Event::Key(key) = evt {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Tab => {
                    state.focus = match state.focus {
                        Panel::Cameras => Panel::Fps,
                        Panel::Fps => Panel::Cameras,
                    };
                }
                KeyCode::BackTab => {
                    state.focus = match state.focus {
                        Panel::Cameras => Panel::Fps,
                        Panel::Fps => Panel::Cameras,
                    };
                }
                KeyCode::Char('?') => {
                    state.status =
                        "Tab: Cameras ↔ FPS · j/k: move · s: spawn viewer · q: quit".into();
                }
                KeyCode::Char('s') => match spawn_selected(&state) {
                    Ok(()) => {
                        state.status = format!(
                            "Launched viewer ({}×{} cells). Press q in that window to close.",
                            DEFAULT_VIEWER_GEOMETRY.cols,
                            DEFAULT_VIEWER_GEOMETRY.rows
                        );
                    }
                    Err(e) => state.status = format!("{e:#}"),
                },
                KeyCode::Char('j') | KeyCode::Down => match state.focus {
                    Panel::Cameras => {
                        let len = state.cameras.len();
                        if len > 0 {
                            let i = state.camera_list.selected().unwrap_or(0);
                            state
                                .camera_list
                                .select(Some((i + 1).min(len - 1)));
                        }
                    }
                    Panel::Fps => {
                        let i = state.fps_list.selected().unwrap_or(0);
                        state.fps_list.select(Some((i + 1).min(3)));
                    }
                },
                KeyCode::Char('k') | KeyCode::Up => match state.focus {
                    Panel::Cameras => {
                        let len = state.cameras.len();
                        if len > 0 {
                            let i = state.camera_list.selected().unwrap_or(0);
                            state.camera_list.select(Some(i.saturating_sub(1)));
                        }
                    }
                    Panel::Fps => {
                        let i = state.fps_list.selected().unwrap_or(0);
                        state.fps_list.select(Some(i.saturating_sub(1)));
                    }
                },
                _ => {}
            }
        }
    }
    Ok(())
}

struct AppState {
    cameras: Vec<(nokhwa::utils::CameraIndex, String)>,
    camera_list: ListState,
    fps_list: ListState,
    focus: Panel,
    status: String,
    fps_choices: [u32; 4],
}

fn fps_value(state: &AppState) -> u32 {
    let i = state.fps_list.selected().unwrap_or(2).min(3);
    state.fps_choices[i]
}

fn spawn_selected(state: &AppState) -> Result<()> {
    let Some(sel) = state.camera_list.selected() else {
        anyhow::bail!("select a camera first");
    };
    let Some((idx, _)) = state.cameras.get(sel) else {
        anyhow::bail!("invalid selection");
    };
    let exe = env::current_exe().context("resolve charviews binary path")?;
    let device = format!("{idx}");
    spawn_viewer(
        &exe,
        Some(&device),
        fps_value(state),
        DEFAULT_VIEWER_GEOMETRY,
    )
}

fn panel_border(focus: Panel, panel: Panel) -> Style {
    if focus == panel {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

fn ui(f: &mut Frame, state: &mut AppState) {
    let area = f.area();

    let logo_block_h = (LOGO_LINES.len() as u16) + 5;
    let keys_h = 4u16;
    let status_h = 5u16;
    let tips_fps_h = 10u16;
    let fixed = logo_block_h + tips_fps_h + keys_h + status_h;
    let cam_h = area.height.saturating_sub(fixed).max(6);

    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(logo_block_h),
            Constraint::Length(cam_h),
            Constraint::Length(tips_fps_h),
            Constraint::Length(keys_h),
            Constraint::Length(status_h),
        ])
        .split(area);

    // --- Logo ---
    let mut logo_text: Vec<Line> = LOGO_LINES
        .iter()
        .map(|ln| {
            Line::from(Span::styled(
                *ln,
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();
    logo_text.push(Line::from(""));
    logo_text.push(Line::from(Span::styled(
        LOGO_TAGLINE,
        Style::default().fg(Color::DarkGray),
    )));
    logo_text.push(Line::from(Span::styled(
        "Command: charviews",
        Style::default().fg(Color::Green),
    )));

    let logo = Paragraph::new(logo_text)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" CharVision ")
                .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(logo, v[0]);

    // --- Cameras (full width) ---
    let cam_items: Vec<ListItem> = state
        .cameras
        .iter()
        .enumerate()
        .map(|(i, (idx, name))| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{i} "), Style::default().fg(Color::Yellow)),
                Span::raw(format!("{idx}")),
                Span::styled(" · ", Style::default().fg(Color::DarkGray)),
                Span::raw(name.as_str()),
            ]))
        })
        .collect();

    let cam_list = List::new(cam_items)
        .block(
            Block::default()
                .title(" Cameras ")
                .borders(Borders::ALL)
                .border_style(panel_border(state.focus, Panel::Cameras)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");
    f.render_stateful_widget(cam_list, v[1], &mut state.camera_list);

    // --- FPS | Omarchy / Hyprland ---
    let low = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(36), Constraint::Percentage(64)])
        .split(v[2]);

    let fps_items: Vec<ListItem> = state
        .fps_choices
        .iter()
        .map(|n| ListItem::new(format!("{n} FPS")))
        .collect();
    let fps_w = List::new(fps_items)
        .block(
            Block::default()
                .title(" Target FPS ")
                .borders(Borders::ALL)
                .border_style(panel_border(state.focus, Panel::Fps)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");
    f.render_stateful_widget(fps_w, low[0], &mut state.fps_list);

    let tips = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Viewer window: ", Style::default().fg(Color::Magenta)),
            Span::raw(format!(
                "{}×{} cells (fixed). ",
                DEFAULT_VIEWER_GEOMETRY.cols,
                DEFAULT_VIEWER_GEOMETRY.rows
            )),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Omarchy / Ghostty: ", Style::default().fg(Color::Yellow)),
            Span::raw("class "),
            Span::styled("TUI.float", Style::default().fg(Color::Green)),
            Span::raw(" → floating-window tag."),
        ]),
        Line::from(vec![
            Span::styled("Dashboard float: ", Style::default().fg(Color::DarkGray)),
            Span::raw("ghostty --class=TUI.float -e charviews"),
        ]),
        Line::from(vec![
            Span::styled("Override: ", Style::default().fg(Color::DarkGray)),
            Span::raw("CHARVISION_FLOAT_CLASS=…"),
        ]),
    ])
    .block(
        Block::default()
            .title(" Launcher ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    )
    .wrap(Wrap { trim: true });
    f.render_widget(tips, low[1]);

    // --- Keys ---
    let keys = Paragraph::new(Line::from(vec![
        Span::styled("Tab ", Style::default().fg(Color::Yellow)),
        Span::raw("Cameras↔FPS  "),
        Span::styled("j/k ", Style::default().fg(Color::Yellow)),
        Span::raw("nav  "),
        Span::styled("s ", Style::default().fg(Color::Green)),
        Span::raw("spawn viewer  "),
        Span::styled("? ", Style::default().fg(Color::Yellow)),
        Span::raw("help  "),
        Span::styled("q ", Style::default().fg(Color::Red)),
        Span::raw("quit"),
    ]))
    .block(
        Block::default()
            .title(" Keys ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(keys, v[3]);

    // --- Status ---
    let status = Paragraph::new(state.status.as_str())
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Status ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(status, v[4]);
}
