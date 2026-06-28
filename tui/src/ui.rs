//! All rendering. Pure function of `&App` → frame; no state mutation here.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{App, Pending, FIELDS, TABS};
use crate::util::{self, group};

const GREEN: Color = Color::Rgb(189, 245, 137);
const BLUE: Color = Color::Rgb(99, 110, 180);
const RED: Color = Color::Rgb(228, 61, 61);
const DIM: Color = Color::Rgb(140, 140, 140);

const BANNER: [&str; 5] = [
    r" ____   ___  _      _          ___   _____     _____  _____  ____    ___  ",
    r"| __ ) |_ _|| |    | |        / _ \ |  ___|   |__  / | ____||  _ \  / _ \ ",
    r"|  _ \  | | | |    | |       | | | || |_        / /  |  _|  | |_) || | | |",
    r"| |_) | | | | |___ | |___    | |_| ||  _|      / /_  | |___ |  _ < | |_| |",
    r"|____/ |___||_____||_____|    \___/ |_|       /____| |_____||_| \_\ \___/ ",
];

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // banner
            Constraint::Length(3), // tabs + flow
            Constraint::Min(8),    // body
            Constraint::Length(3), // on-chain status
            Constraint::Length(6), // activity log
        ])
        .split(f.area());

    render_banner(f, chunks[0]);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(34), Constraint::Min(20)])
        .split(chunks[1]);
    render_tabs(f, app, top[0]);
    render_flow(f, app, top[1]);

    match app.tab {
        0 => render_buyer(f, app, chunks[2]),
        1 => render_seller(f, app, chunks[2]),
        _ => render_auditor(f, app, chunks[2]),
    }
    render_status(f, app, chunks[3]);
    render_log(f, app, chunks[4]);

    if let Some(p) = app.confirm {
        render_confirm(f, p);
    }
    if app.show_help {
        render_help(f);
    }
}

fn render_banner(f: &mut Frame, area: Rect) {
    let mut lines: Vec<Line> = BANNER
        .iter()
        .map(|l| Line::from(Span::styled(*l, Style::default().fg(GREEN).add_modifier(Modifier::BOLD))).alignment(Alignment::Center))
        .collect();
    lines.push(Line::from(Span::styled("Zero-knowledge Letter-of-Credit settlement on Stellar", Style::default().fg(DIM))).alignment(Alignment::Center));
    f.render_widget(Paragraph::new(lines), area);
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(" LC #{} · {} ", app.cfg.deployment.lc_id, app.cfg.deployment.network);
    let tabs = Tabs::new(TABS.iter().map(|t| Line::from(format!(" {t} "))).collect::<Vec<_>>())
        .block(Block::default().borders(Borders::ALL).title(title).border_style(Style::default().fg(BLUE)))
        .select(app.tab)
        .highlight_style(Style::default().fg(Color::Black).bg(GREEN).add_modifier(Modifier::BOLD))
        .divider("│");
    f.render_widget(tabs, area);
}

/// ① Fund → ② Prove → ③ Release → ④ Refund, lit by on-chain state.
fn render_flow(f: &mut Frame, app: &App, area: Rect) {
    let escrow_pos = app.status.as_ref().map_or(false, |s| s.escrow_bal.trim().parse::<i128>().unwrap_or(0) > 0);
    let escrow_zero = app.status.as_ref().map_or(false, |s| s.escrow_bal.trim().parse::<i128>().unwrap_or(0) == 0);
    let released = app.status.as_ref().map_or(false, |s| s.released);
    let steps = [
        ("① Fund", escrow_pos),
        ("② Prove", app.proof.is_some()),
        ("③ Release", released),
        ("④ Refund", released && escrow_zero),
    ];
    let mut spans = Vec::new();
    for (i, (label, done)) in steps.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" → ", Style::default().fg(DIM)));
        }
        let style = if *done {
            Style::default().fg(GREEN).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(DIM)
        };
        spans.push(Span::styled(*label, style));
    }
    f.render_widget(
        Paragraph::new(Line::from(spans).alignment(Alignment::Center))
            .block(Block::default().borders(Borders::ALL).title(" Flow ").border_style(Style::default().fg(DIM))),
        area,
    );
}

fn panel(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(format!(" {title} "), Style::default().fg(GREEN).add_modifier(Modifier::BOLD)))
        .border_style(Style::default().fg(DIM))
}

fn render_buyer(f: &mut Frame, app: &App, area: Rect) {
    let t = &app.cfg.terms;
    let lines = vec![
        Line::from(vec![Span::styled("Role: ", Style::default().fg(DIM)), Span::styled(format!("BUYER · {}", t.buyer), Style::default().fg(BLUE).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from("The buyer locks funds in escrow, reclaims any remainder after"),
        Line::from("settlement, and can cancel for a refund once the LC expires."),
        Line::from(""),
        Line::from(vec![Span::styled("Credit limit: ", Style::default().fg(DIM)), Span::raw(format!("{} USDC", group(&t.credit_limit_usdc.to_string())))]),
        Line::from(vec![Span::styled("Escrow:       ", Style::default().fg(DIM)), Span::raw(&app.cfg.deployment.escrow)]),
        Line::from(""),
        Line::from(vec![
            Span::styled("▸ Fund amount (USDC)   ", Style::default().fg(Color::White)),
            Span::styled(format!("{}▏", app.fund_amount), Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![key("f"), Span::raw(" Fund    "), key("x"), Span::raw(" Refund to buyer    "), key("s"), Span::raw(" Refresh")]),
    ];
    f.render_widget(Paragraph::new(lines).block(panel("Buyer")).wrap(Wrap { trim: true }), area);
}

fn render_seller(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: document inputs + the live rule checklist.
    let mut left: Vec<Line> = vec![
        Line::from(vec![Span::styled("Role: ", Style::default().fg(DIM)), Span::styled(format!("SELLER · {}", app.cfg.terms.seller), Style::default().fg(GREEN).add_modifier(Modifier::BOLD))]),
        Line::from(""),
    ];
    for (i, label) in FIELDS.iter().enumerate() {
        let focused = i == app.field_idx;
        let val = &app.fields[i];
        let val_span = if focused {
            Span::styled(format!("{val}▏"), Style::default().fg(GREEN).add_modifier(Modifier::BOLD))
        } else {
            Span::raw(val.clone())
        };
        left.push(Line::from(vec![
            Span::styled(if focused { "▸ " } else { "  " }, Style::default().fg(GREEN)),
            Span::styled(format!("{label:<24}"), Style::default().fg(if focused { Color::White } else { DIM })),
            val_span,
        ]));
    }
    left.push(Line::from(""));
    let amount: Option<u64> = app.fields[0].trim().parse().ok();
    let ship = util::ymd_to_unix(&app.fields[1]).ok();
    let balance: Option<u64> = app.fields[2].trim().parse().ok();
    let limit = app.cfg.terms.credit_limit_usdc;
    let deadline = app.cfg.terms.shipment_deadline_unix;
    left.push(Line::from(Span::styled("Rules enforced in the zkVM guest:", Style::default().fg(DIM))));
    left.push(rule("amount ≤ credit limit", amount.map(|a| a <= limit)));
    left.push(rule("ship date ≤ deadline", ship.map(|s| s <= deadline)));
    left.push(rule("seller ∈ approved set", Some(true)));
    left.push(rule("buyer balance ≥ amount", match (balance, amount) { (Some(b), Some(a)) => Some(b >= a), _ => None }));
    left.push(rule("issuer ed25519 signature", Some(true)));
    f.render_widget(Paragraph::new(left).block(panel("Seller · documents")), cols[0]);

    // Right: proof output / live progress + controls.
    let proving = app.busy.as_deref() == Some("prove");
    let mut right: Vec<Line> = Vec::new();
    if proving {
        let elapsed = app.elapsed_secs().unwrap_or(0);
        right.push(Line::from(vec![
            Span::styled(format!("{} proving… ", app.spinner()), Style::default().fg(GREEN)),
            Span::styled(format!("{}:{:02}", elapsed / 60, elapsed % 60), Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
        ]));
        right.push(Line::from(Span::styled("real Groth16 in the RISC Zero zkVM + Docker wrap", Style::default().fg(DIM))));
        right.push(Line::from(""));
        if let Some(p) = &app.proof_progress {
            right.push(Line::from(Span::styled(trunc_n(p, 46), Style::default().fg(BLUE))));
        }
    } else if let Some(p) = &app.proof {
        right.push(Line::from(vec![Span::styled("Real Groth16 proof", Style::default().fg(GREEN).add_modifier(Modifier::BOLD))]));
        right.push(Line::from(""));
        right.push(field("lc_id", &p.lc_id));
        right.push(field("terms_digest", &trunc(&p.terms_digest)));
        right.push(field("disclosure_cmt", &trunc(&p.disclosure_cmt)));
        right.push(field("journal", &trunc(&p.journal)));
        right.push(Line::from(vec![Span::styled("seal          ", Style::default().fg(DIM)), Span::styled(trunc(&p.seal), Style::default().fg(GREEN).add_modifier(Modifier::BOLD))]));
        let (msg, color) = if p.seal.starts_with("73c457ba") {
            ("✓ selector 73c457ba — verifiable on-chain", GREEN)
        } else {
            ("⚠ unexpected seal selector", RED)
        };
        right.push(Line::from(Span::styled(msg, Style::default().fg(color))));
    } else {
        right.push(Line::from(Span::styled("No proof yet.", Style::default().fg(DIM))));
        right.push(Line::from(Span::styled("Type the documents, then press [p].", Style::default().fg(DIM))));
    }
    right.push(Line::from(""));
    right.push(Line::from(vec![key("↑/↓"), Span::raw(" field  "), key("p"), Span::raw(" prove  "), key("r"), Span::raw(" release")]));
    f.render_widget(Paragraph::new(right).block(panel("Seller · proof")).wrap(Wrap { trim: true }), cols[1]);
}

fn render_auditor(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![
        Line::from(vec![Span::styled("Role: ", Style::default().fg(DIM)), Span::styled("AUDITOR", Style::default().fg(BLUE).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from("The invoice amount never went on-chain. The auditor holds the"),
        Line::from("view key, opens the disclosure, and checks it matches the"),
        Line::from("commitment the escrow recorded."),
        Line::from(""),
    ];
    if let Some(d) = &app.disclosure {
        lines.push(field("invoice amount", &group(&d.amount)));
        lines.push(field("buyer balance", &group(&d.balance)));
        lines.push(field("ship date (unix)", &d.ship_date));
        lines.push(field("buyer_id", &trunc(&d.buyer_id)));
        lines.push(field("seller_id", &trunc(&d.seller_id)));
        lines.push(field("commitment", &trunc(&d.commitment)));
        lines.push(Line::from(Span::styled("must equal escrow.disclosure() on-chain", Style::default().fg(DIM))));
    } else {
        lines.push(Line::from(Span::styled("Disclosure not opened yet.", Style::default().fg(DIM))));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![key("a"), Span::raw(" Decrypt disclosure")]));
    f.render_widget(Paragraph::new(lines).block(panel("Auditor")).wrap(Wrap { trim: true }), area);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let badge = match &app.status {
        Some(s) if s.released => Span::styled(" ● RELEASED ", Style::default().fg(Color::Black).bg(GREEN)),
        Some(_) => Span::styled(" ● FUNDED ", Style::default().fg(Color::Black).bg(BLUE)),
        None => Span::styled(" … ", Style::default().fg(DIM)),
    };
    let (escrow, seller) = match &app.status {
        Some(s) => (group(&s.escrow_bal), group(&s.seller_bal)),
        None => ("…".into(), "…".into()),
    };
    let busy = match &app.busy {
        Some(b) => {
            let e = app.elapsed_secs().unwrap_or(0);
            Span::styled(format!("{} {b} {}:{:02}", app.spinner(), e / 60, e % 60), Style::default().fg(GREEN))
        }
        None => Span::styled("idle", Style::default().fg(DIM)),
    };
    let line = Line::from(vec![
        Span::styled("escrow ", Style::default().fg(DIM)), Span::raw(escrow),
        Span::raw("   "), Span::styled("seller ", Style::default().fg(DIM)), Span::raw(seller),
        Span::raw("   "), badge,
        Span::raw("   "), Span::styled("key:", Style::default().fg(DIM)), Span::raw(app.source.clone()),
        Span::raw("   "), busy,
    ]);
    let mut block = panel("On-chain");
    if let Some(h) = &app.last_tx {
        block = block.title_bottom(Line::from(Span::styled(format!(" stellar.expert/explorer/testnet/tx/{h} "), Style::default().fg(BLUE))).right_aligned());
    }
    f.render_widget(Paragraph::new(line).block(block), area);
}

fn render_log(f: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = app.log.iter().map(|l| {
        let c = if l.starts_with('✓') { GREEN } else if l.starts_with('✗') { RED } else { DIM };
        Line::from(Span::styled(l.clone(), Style::default().fg(c)))
    }).collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Activity ")
        .title_bottom(Line::from(" Tab role · ? help · Esc quit ").right_aligned())
        .border_style(Style::default().fg(DIM));
    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_confirm(f: &mut Frame, p: Pending) {
    let area = centered_rect(60, 20, f.area());
    f.render_widget(Clear, area);
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(p.prompt(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD))).alignment(Alignment::Center),
        Line::from(""),
        Line::from(vec![
            Span::styled("[y]", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
            Span::raw(" yes    "),
            Span::styled("[any other key]", Style::default().fg(DIM)),
            Span::raw(" cancel"),
        ]).alignment(Alignment::Center),
    ];
    f.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Confirm ").border_style(Style::default().fg(RED))),
        area,
    );
}

fn render_help(f: &mut Frame) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);
    let row = |k: &'static str, d: &'static str| Line::from(vec![Span::styled(format!("  {k:<10}"), Style::default().fg(GREEN)), Span::styled(d, Style::default().fg(Color::White))]);
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("  Global", Style::default().fg(BLUE).add_modifier(Modifier::BOLD))),
        row("Tab", "switch role (Buyer / Seller / Auditor)"),
        row("?", "toggle this help"),
        row("Esc / q", "quit"),
        Line::from(""),
        Line::from(Span::styled("  Buyer", Style::default().fg(BLUE).add_modifier(Modifier::BOLD))),
        row("0-9", "edit fund amount"),
        row("f", "fund the escrow"),
        row("x", "refund remainder to buyer"),
        row("s", "refresh on-chain status"),
        Line::from(""),
        Line::from(Span::styled("  Seller", Style::default().fg(BLUE).add_modifier(Modifier::BOLD))),
        row("↑/↓", "move between document fields"),
        row("type", "edit the focused field"),
        row("p / Enter", "generate real Groth16 proof"),
        row("r", "release escrow with the proof"),
        Line::from(""),
        Line::from(Span::styled("  Auditor", Style::default().fg(BLUE).add_modifier(Modifier::BOLD))),
        row("a", "decrypt the disclosure"),
        Line::from(""),
        Line::from(Span::styled("  press any key to close", Style::default().fg(DIM))).alignment(Alignment::Center),
    ];
    f.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Keys ").border_style(Style::default().fg(GREEN))),
        area,
    );
}

// --- helpers --------------------------------------------------------------

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(v[1])[1]
}

fn key(k: &str) -> Span<'static> {
    Span::styled(format!("[{k}]"), Style::default().fg(GREEN).add_modifier(Modifier::BOLD))
}

fn field(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<14}"), Style::default().fg(DIM)),
        Span::raw(value.to_string()),
    ])
}

fn rule(label: &str, ok: Option<bool>) -> Line<'static> {
    let (mark, color) = match ok {
        Some(true) => ("✔", GREEN),
        Some(false) => ("✗", RED),
        None => ("?", DIM),
    };
    Line::from(vec![
        Span::styled(format!("  {mark} "), Style::default().fg(color)),
        Span::styled(label.to_string(), Style::default().fg(if ok == Some(false) { RED } else { DIM })),
    ])
}

fn trunc(s: &str) -> String {
    trunc_n(s, 24)
}

fn trunc_n(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let head: String = s.chars().take(n).collect();
        format!("{head}…")
    }
}
