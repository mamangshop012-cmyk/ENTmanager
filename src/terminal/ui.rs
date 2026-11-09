use std::{
    io::{self, Stdout},
    time::Duration,
};

use chrono::Local;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

use tokio::sync::mpsc::Receiver;

use super::{
    event::{DashboardEvent, Feature, LogKind},
    state::{DashboardState, FeatureStatus},
};

type TerminalBackend = CrosstermBackend<Stdout>;

type DashboardTerminal = Terminal<TerminalBackend>;

const CARD_HEIGHT: u16 = 3;
const CARD_GAP: u16 = 1;

// ============================================================================
// COLORS
// ============================================================================

fn status_background(status: FeatureStatus) -> Color {
    match status {
        // Tidak aktif = merah gelap
        FeatureStatus::Idle => Color::Rgb(92, 48, 10),

        // Sedang aktif = hijau
        FeatureStatus::Active => Color::Rgb(18, 100, 45),

        // Baru selesai
        FeatureStatus::Success => Color::Rgb(15, 75, 90),

        // Menunggu
        FeatureStatus::Waiting => Color::Rgb(100, 75, 18),

        // Error
        FeatureStatus::Error => Color::Rgb(145, 18, 18),
    }
}

fn status_foreground(status: FeatureStatus) -> Color {
    match status {
        FeatureStatus::Idle => Color::Rgb(255, 238, 210),

        FeatureStatus::Active => Color::Rgb(225, 255, 232),

        FeatureStatus::Success => Color::Rgb(225, 250, 255),

        FeatureStatus::Waiting => Color::Rgb(255, 245, 210),

        FeatureStatus::Error => Color::White,
    }
}

fn status_border(status: FeatureStatus) -> Color {
    match status {
        FeatureStatus::Idle => Color::Rgb(215, 125, 25),

        FeatureStatus::Active => Color::Rgb(35, 175, 75),

        FeatureStatus::Success => Color::Rgb(25, 130, 155),

        FeatureStatus::Waiting => Color::Rgb(180, 130, 20),

        FeatureStatus::Error => Color::Red,
    }
}

// ============================================================================
// TIME
// ============================================================================

fn current_clock() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

fn current_date() -> String {
    Local::now().format("%d %b %Y").to_string()
}

// ============================================================================
// HELPERS
// ============================================================================

fn truncate(text: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }

    if text.chars().count() <= max {
        return text.to_string();
    }

    if max == 1 {
        return "…".to_string();
    }

    let mut result: String = text.chars().take(max - 1).collect();

    result.push('…');

    result
}

// ============================================================================
// HEADER
// ============================================================================

fn render_header(frame: &mut Frame<'_>, area: Rect) {
    let line = Line::from(vec![
        Span::styled(
            " MANAGERRUSTENT ",
            Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(28, 28, 36))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "• LIVE ENGINE ",
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::Rgb(28, 28, 36))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("• {}", current_clock()),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(line).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        ),
        area,
    );
}

// ============================================================================
// FEATURE CARD
// ============================================================================

fn render_feature_card(
    frame: &mut Frame<'_>,
    area: Rect,
    feature: Feature,
    state: &super::state::FeatureState,
) {
    if area.width < 10 || area.height < CARD_HEIGHT {
        return;
    }

    let bg = status_background(state.status);

    let fg = status_foreground(state.status);

    let border = status_border(state.status);

    let inner_width = area.width.saturating_sub(2) as usize;

    let title = truncate(feature.as_str(), inner_width);

    let detail = if state.last_detail.is_empty() {
        state.status.label().to_string()
    } else {
        format!("{} • {}", state.status.label(), state.last_detail)
    };

    let detail = truncate(&detail, inner_width);

    let calls = format!("CALLS x{}", state.count);

    let lines = vec![
        Line::from(Span::styled(
            format!("{:^width$}", title, width = inner_width),
            Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("{:^width$}", detail, width = inner_width),
            Style::default().fg(fg).bg(bg),
        )),
        Line::from(Span::styled(
            format!("{:^width$}", calls, width = inner_width),
            Style::default().fg(fg).bg(bg),
        )),
    ];

    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border))
                .style(Style::default().bg(bg)),
        ),
        area,
    );
}

// ============================================================================
// FEATURE MATRIX
// ============================================================================

fn render_feature_matrix(frame: &mut Frame<'_>, area: Rect, state: &DashboardState) {
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    if inner.width < 20 || inner.height < 8 {
        frame.render_widget(
            Paragraph::new("Terminal terlalu kecil.\nPerbesar window.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow)),
            inner,
        );

        return;
    }

    let columns = if inner.width >= 118 {
        6
    } else if inner.width >= 96 {
        5
    } else if inner.width >= 76 {
        4
    } else if inner.width >= 56 {
        3
    } else {
        2
    };

    let feature_order = feature_order();

    let rows = (feature_order.len() + columns - 1) / columns;

    let card_width =
        (inner.width.saturating_sub((columns - 1) as u16 * CARD_GAP) / columns as u16).max(14);

    for row in 0..rows {
        for col in 0..columns {
            let index = row * columns + col;

            if index >= feature_order.len() {
                continue;
            }

            let feature = feature_order[index];

            let x = inner.x + col as u16 * (card_width + CARD_GAP);

            let y = inner.y + row as u16 * (CARD_HEIGHT + CARD_GAP);

            if y + CARD_HEIGHT > inner.y + inner.height {
                continue;
            }

            let card = Rect {
                x,
                y,
                width: card_width,
                height: CARD_HEIGHT,
            };

            let feature_state = state.features.get(&feature).cloned().unwrap_or_default();

            render_feature_card(frame, card, feature, &feature_state);
        }
    }
}

// ============================================================================
// ORDER FEATURE
// ============================================================================
//
// Urutan ini dibuat konsisten dengan dashboard dummy kita.
// ============================================================================

fn feature_order() -> &'static [Feature] {
    &[
        Feature::Start,
        Feature::Ping,
        Feature::Ban,
        Feature::Tban,
        Feature::Unban,
        Feature::Mute,
        Feature::Tmute,
        Feature::Unmute,
        Feature::Kick,
        Feature::Kickme,
        Feature::Warn,
        Feature::Warns,
        Feature::Resetwarn,
        Feature::Resetallwarn,
        Feature::Setwarn,
        Feature::Warnmode,
        Feature::Cleanwarn,
        Feature::Lock,
        Feature::Unlock,
        Feature::Locks,
        Feature::Locktypes,
        Feature::Lockwarns,
        Feature::Allowlist,
        Feature::Rmallowlist,
        Feature::Rmallowlistall,
        Feature::Addbl,
        Feature::Delbl,
        Feature::Blacklist,
        Feature::Setlog,
        Feature::Setslot,
        Feature::Addbl1,
        Feature::Addbl2,
        Feature::Addbl3,
        Feature::Addbl4,
        Feature::Addbl5,
        Feature::Filter,
        Feature::Filters,
        Feature::Stop,
        Feature::Stopall,
        Feature::Welcome,
        Feature::Setwelcome,
        Feature::Cleanwelcome,
        Feature::Addadmin,
        Feature::Demote,
        Feature::Demoteme,
        Feature::Modstatus,
        Feature::Callback,
        Feature::Database,
        Feature::TelegramApi,
        Feature::Metrics,
    ]
}

// ============================================================================
// SYSTEM PANEL
// ============================================================================

fn render_system_panel(frame: &mut Frame<'_>, area: Rect, state: &DashboardState) {
    let metrics = &state.metrics;

    let telegram = match metrics.telegram_rtt_ms {
        Some(ms) => format!("{} ms", ms),

        None => "N/A".to_string(),
    };

    let database = match metrics.database_rtt_us {
        Some(us) => format!("{} us", us),

        None => "N/A".to_string(),
    };

    let internal = match metrics.internal_rtt_us {
        Some(us) => format!("{} us", us),

        None => "N/A".to_string(),
    };

    let cpu = metrics
        .cpu_percent
        .map(|v| format!("{:.1}%", v))
        .unwrap_or_else(|| "N/A".to_string());

    let ram = metrics
        .ram_percent
        .map(|v| format!("{:.1}%", v))
        .unwrap_or_else(|| "N/A".to_string());

    let lines = vec![
        Line::from(format!("CLOCK         {}", current_clock())),
        Line::from(format!("DATE          {}", current_date())),
        Line::from(format!("UPTIME        {}s", metrics.uptime_seconds)),
        Line::from(""),
        Line::from(vec![
            Span::styled("TELEGRAM RTT  ", Style::default().fg(Color::Gray)),
            Span::styled(
                telegram,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(format!("DATABASE RTT  {}", database)),
        Line::from(format!("INTERNAL      {}", internal)),
        Line::from(format!("CPU           {}", cpu)),
        Line::from(format!("RAM           {}", ram)),
        Line::from(format!("GROUPS        {}", metrics.groups)),
        Line::from(format!("USERS         {}", metrics.users)),
        Line::from(format!("ACTIONS       {}", metrics.total_actions)),
        Line::from(format!("ERRORS        {}", metrics.errors)),
    ];

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" SYSTEM / PERFORMANCE ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}

// ============================================================================
// LAST ACTION
// ============================================================================

fn render_last_action(frame: &mut Frame<'_>, area: Rect, state: &DashboardState) {
    let text = state.last_action.as_deref().unwrap_or("No activity yet");

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                "LAST ACTIVITY",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(truncate(text, area.width.saturating_sub(2) as usize)),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        ),
        area,
    );
}

// ============================================================================
// LAST ERROR
// ============================================================================

fn render_last_error(frame: &mut Frame<'_>, area: Rect, state: &DashboardState) {
    let text = state.last_error.as_deref().unwrap_or("No errors");

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                "LAST ERROR",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(truncate(text, area.width.saturating_sub(2) as usize)),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        ),
        area,
    );
}

// ============================================================================
// LIVE LOG
// ============================================================================

fn log_color(kind: LogKind) -> Color {
    match kind {
        LogKind::System => Color::Cyan,

        LogKind::Action => Color::Green,

        LogKind::Api => Color::Blue,

        LogKind::Database => Color::Cyan,

        LogKind::Callback => Color::Magenta,

        LogKind::Warn => Color::Yellow,

        LogKind::Error => Color::Red,

        LogKind::Debug => Color::DarkGray,
    }
}

fn render_logs(frame: &mut Frame<'_>, area: Rect, state: &DashboardState) {
    let mut lines = Vec::new();

    for log in state.logs.iter() {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<9}", log.kind.as_str()),
                Style::default()
                    .fg(log_color(log.kind))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                truncate(&log.message, area.width.saturating_sub(12) as usize),
                Style::default().fg(Color::White),
            ),
        ]));
    }

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(
            Block::default()
                .title(" LIVE EVENT STREAM ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        ),
        area,
    );
}

// ============================================================================
// FOOTER
// ============================================================================

fn render_footer(frame: &mut Frame<'_>, area: Rect) {
    let line = Line::from(vec![
        Span::styled(
            " IDLE ",
            Style::default()
                .fg(Color::White)
                .bg(status_background(FeatureStatus::Idle)),
        ),
        Span::raw(" "),
        Span::styled(
            " ACTIVE ",
            Style::default()
                .fg(Color::White)
                .bg(status_background(FeatureStatus::Active)),
        ),
        Span::raw(" "),
        Span::styled(
            " SUCCESS ",
            Style::default()
                .fg(Color::White)
                .bg(status_background(FeatureStatus::Success)),
        ),
        Span::raw(" "),
        Span::styled(
            " ERROR ",
            Style::default()
                .fg(Color::White)
                .bg(status_background(FeatureStatus::Error)),
        ),
        Span::styled(
            "       Q / ESC = EXIT",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    frame.render_widget(
        Paragraph::new(line).alignment(Alignment::Center).block(
            Block::default()
                .title(" STATUS ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        ),
        area,
    );
}

// ============================================================================
// DRAW
// ============================================================================

fn draw(terminal: &mut DashboardTerminal, state: &DashboardState) -> io::Result<()> {
    terminal.draw(|frame| {
        let root = frame.area();

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(root);

        // HEADER
        render_header(frame, vertical[0]);

        // MAIN
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(vertical[1]);

        // ============================================================
        // LEFT
        // ============================================================

        let left = horizontal[0];

        frame.render_widget(
            Block::default()
                .title(" FEATURE MATRIX ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
            left,
        );

        render_feature_matrix(frame, left, state);

        // ============================================================
        // RIGHT
        // ============================================================

        let right = horizontal[1];

        let right_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(15),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Min(5),
            ])
            .split(right);

        render_system_panel(frame, right_layout[0], state);

        render_last_action(frame, right_layout[1], state);

        render_last_error(frame, right_layout[2], state);

        render_logs(frame, right_layout[3], state);

        // FOOTER
        render_footer(frame, vertical[2]);
    })?;

    Ok(())
}

// ============================================================================
// TERMINAL SETUP
// ============================================================================

fn setup_terminal() -> io::Result<DashboardTerminal> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);

    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut DashboardTerminal) -> io::Result<()> {
    disable_raw_mode()?;

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    terminal.show_cursor()?;

    Ok(())
}

// ============================================================================
// REAL DASHBOARD WORKER
// ============================================================================

pub async fn run(mut receiver: Receiver<DashboardEvent>) -> io::Result<()> {
    let mut terminal = setup_terminal()?;

    let mut state = DashboardState::default();

    let result = run_loop(&mut terminal, &mut state, &mut receiver).await;

    let restore_result = restore_terminal(&mut terminal);

    match result {
        Err(error) => Err(error),

        Ok(()) => restore_result,
    }
}

// ============================================================================
// LOOP
// ============================================================================

async fn run_loop(
    terminal: &mut DashboardTerminal,

    state: &mut DashboardState,

    receiver: &mut Receiver<DashboardEvent>,
) -> io::Result<()> {
    let mut ticker = tokio::time::interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            maybe_event =
                receiver.recv() =>
            {
                match maybe_event {
                    Some(event) => {
                        state.apply_event(event);
                    }

                    None => {
                        break;
                    }
                }
            }

            _ =
                ticker.tick() =>
            {
                // Input keyboard hanya untuk dashboard.
                if event::poll(
                    Duration::from_millis(0)
                )? {
                    if let Event::Key(key) =
                        event::read()?
                    {
                        match key.code {
                            KeyCode::Char('q')
                            | KeyCode::Esc => {
                                break;
                            }

                            _ => {}
                        }
                    }
                }

                state.tick();

                draw(
                    terminal,
                    state,
                )?;
            }
        }
    }

    Ok(())
}
