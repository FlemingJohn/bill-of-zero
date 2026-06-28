//! All rendering. Pure function of `&App` → frame; no state mutation here.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{App, FIELDS, TABS};
use crate::util::{self, group};

const GREEN: Color = Color::Rgb(189, 245, 137);
const BLUE: Color = Color::Rgb(99, 110, 180);
const RED: Color = Color::Rgb(228, 61, 61);
const DIM: Color = Color::Rgb(140, 140, 140);

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tabs
            Constraint::Min(8),    // body
            Constraint::Length(3), // on-chain status
            Constraint::Length(8), // activity log
        ])
        .split(f.area());

    render_tabs(f, app, chunks[0]);
    match app.tab {
        0 => render_buyer(f, app, chunks[1]),
        1 => render_seller(f, app, chunks[1]),
        _ => render_auditor(f, app, chunks[1]),
    }
    render_status(f, app, chunks[2]);
    render_log(f, app, chunks[3]);
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(
        " Bill of Zero — LC #{} · {} ",
        app.cfg.deployment.lc_id, app.cfg.deployment.network
    );
    let tabs = Tabs::new(TABS.iter().map(|t| Line::from(format!(" {t} "))).collect::<Vec<_>>())
        .block(Block::default().borders(Borders::ALL).title(title).border_style(Style::default().fg(BLUE)))
        .select(app.tab)
        .highlight_style(Style::default().fg(Color::Black).bg(GREEN).add_modifier(Modifier::BOLD))
        .divider("│");
    f.render_widget(tabs, area);
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
        Line::from(format!("The buyer opens the LC and locks funds in escrow until the")),
        Line::from(format!("seller proves the shipment documents comply.")),
        Line::from(""),
        Line::from(vec![Span::styled("Credit limit: ", Style::default().fg(DIM)), Span::raw(format!("{} USDC", group(&t.credit_limit_usdc.to_string())))]),
        Line::from(vec![Span::styled("Escrow:       ", Style::default().fg(DIM)), Span::raw(&app.cfg.deployment.escrow)]),
        Line::from(""),
        Line::from(vec![key("f"), Span::raw(" Fund escrow +100,000    "), key("s"), Span::raw(" Refresh on-chain status")]),
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
        let marker = if focused { "▸ " } else { "  " };
        left.push(Line::from(vec![
            Span::styled(marker, Style::default().fg(GREEN)),
            Span::styled(format!("{label:<24}"), Style::default().fg(if focused { Color::White } else { DIM })),
            val_span,
        ]));
    }
    left.push(Line::from(""));

    // Live (illustrative) rule status from the typed values.
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

    // Right: proof output + controls.
    let mut right: Vec<Line> = vec![
        Line::from(vec![Span::styled("Real Groth16 proof", Style::default().fg(GREEN).add_modifier(Modifier::BOLD)), Span::styled("  · takes a few minutes", Style::default().fg(DIM))]),
        Line::from(""),
    ];
    if let Some(p) = &app.proof {
        right.push(field("lc_id", &p.lc_id));
        right.push(field("terms_digest", &trunc(&p.terms_digest)));
        right.push(field("disclosure_cmt", &trunc(&p.disclosure_cmt)));
        right.push(field("journal", &trunc(&p.journal)));
        right.push(Line::from(vec![Span::styled("seal          ", Style::default().fg(DIM)), Span::styled(trunc(&p.seal), Style::default().fg(GREEN).add_modifier(Modifier::BOLD))]));
        let (msg, color) = if p.seal.starts_with("73c457ba") {
            ("✓ real Groth16 selector 73c457ba — verifiable on-chain", GREEN)
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
        Line::from("view key and can selectively open the disclosure, then check it"),
        Line::from("matches the commitment the escrow recorded."),
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
        Some(b) => Span::styled(format!("{} {b}…", app.spinner()), Style::default().fg(GREEN)),
        None => Span::styled("idle", Style::default().fg(DIM)),
    };
    let line = Line::from(vec![
        Span::styled("escrow ", Style::default().fg(DIM)), Span::raw(escrow),
        Span::raw("   "), Span::styled("seller ", Style::default().fg(DIM)), Span::raw(seller),
        Span::raw("   "), badge,
        Span::raw("   "), Span::styled("key:", Style::default().fg(DIM)), Span::raw(app.source.clone()),
        Span::raw("   "), busy,
    ]);
    f.render_widget(Paragraph::new(line).block(panel("On-chain")), area);
}

fn render_log(f: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = app.log.iter().map(|l| {
        let c = if l.starts_with('✓') { GREEN } else if l.starts_with('✗') { RED } else { DIM };
        Line::from(Span::styled(l.clone(), Style::default().fg(c)))
    }).collect();
    let help = Block::default()
        .borders(Borders::ALL)
        .title(" Activity ")
        .title_bottom(Line::from(" Tab switch role · Esc quit ").right_aligned())
        .border_style(Style::default().fg(DIM));
    f.render_widget(Paragraph::new(lines).block(help), area);
}

// --- small span builders --------------------------------------------------

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
    if s.len() <= 24 {
        s.to_string()
    } else {
        format!("{}…", &s[..24])
    }
}
