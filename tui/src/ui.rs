//! All rendering. Pure function of `&App` → frame; no state mutation here.

use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{App, Pending, BUYER_FIELDS, PROFILES, SELLER_FIELDS};
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
    let tabs = Tabs::new(crate::app::TABS.iter().map(|t| Line::from(format!(" {t} "))).collect::<Vec<_>>())
        .block(Block::default().borders(Borders::ALL).title(title).border_style(Style::default().fg(BLUE)))
        .select(app.tab)
        .highlight_style(Style::default().fg(Color::Black).bg(GREEN).add_modifier(Modifier::BOLD))
        .divider("│");
    f.render_widget(tabs, area);
}

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
    let mut lines = vec![
        Line::from(vec![Span::styled("Role: ", Style::default().fg(DIM)), Span::styled(format!("BUYER · {}", t.buyer), Style::default().fg(BLUE).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from("The buyer locks funds in escrow and owns the private balance"),
        Line::from("used by the range proof. Reclaims the remainder after settlement."),
        Line::from(""),
    ];
    for (i, label) in BUYER_FIELDS.iter().enumerate() {
        let focused = i == app.buyer_idx;
        let val = &app.buyer_fields[i];
        let val_span = if focused {
            Span::styled(format!("{val}▏"), Style::default().fg(GREEN).add_modifier(Modifier::BOLD))
        } else {
            Span::raw(val.clone())
        };
        lines.push(Line::from(vec![
            Span::styled(if focused { "▸ " } else { "  " }, Style::default().fg(GREEN)),
            Span::styled(format!("{label:<22}"), Style::default().fg(if focused { Color::White } else { DIM })),
            val_span,
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("Credit limit: ", Style::default().fg(DIM)), Span::raw(format!("{} {}", group(&t.credit_limit_usdc.to_string()), t.currency))]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![key("↑/↓"), Span::raw(" field  "), key("f"), Span::raw(" Fund  "), key("x"), Span::raw(" Refund  "), key("s"), Span::raw(" Refresh")]));
    f.render_widget(Paragraph::new(lines).block(panel("Buyer")).wrap(Wrap { trim: true }), area);
}

fn render_seller(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(area);

    // Left: document inputs + derived amount + the live rule checklist.
    let mut left: Vec<Line> = vec![
        Line::from(vec![Span::styled("Role: ", Style::default().fg(DIM)), Span::styled(format!("SELLER · {}", app.cfg.terms.seller), Style::default().fg(GREEN).add_modifier(Modifier::BOLD))]),
    ];
    for (i, label) in SELLER_FIELDS.iter().enumerate() {
        let focused = i == app.seller_idx;
        let val = &app.seller_fields[i];
        let val_span = if focused {
            Span::styled(format!("{val}▏"), Style::default().fg(GREEN).add_modifier(Modifier::BOLD))
        } else {
            Span::raw(val.clone())
        };
        left.push(Line::from(vec![
            Span::styled(if focused { "▸ " } else { "  " }, Style::default().fg(GREEN)),
            Span::styled(format!("{label:<22}"), Style::default().fg(if focused { Color::White } else { DIM })),
            val_span,
        ]));
    }
    let amount = app.derived_amount();
    left.push(Line::from(vec![
        Span::styled("  = invoice amount     ", Style::default().fg(DIM)),
        Span::styled(amount.map(|a| group(&a.to_string())).unwrap_or_else(|| "?".into()), Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
    ]));
    f.render_widget(Paragraph::new(left).block(panel("Seller · documents")), cols[0]);

    // Right: rule checklist + proof / live progress + controls.
    let ship = util::ymd_to_unix(&app.seller_fields[2]).ok();
    let balance: Option<u64> = app.buyer_fields[1].trim().parse().ok();
    let limit = app.cfg.terms.credit_limit_usdc;
    let deadline = app.cfg.terms.shipment_deadline_unix;
    let cur_match = app.seller_fields[3].trim().eq_ignore_ascii_case(app.cfg.terms.currency.trim());

    let mut right: Vec<Line> = Vec::new();
    let proving = app.busy.as_deref() == Some("prove");
    if proving {
        let e = app.elapsed_secs().unwrap_or(0);
        right.push(Line::from(vec![
            Span::styled(format!("{} proving… ", app.spinner()), Style::default().fg(GREEN)),
            Span::styled(format!("{}:{:02}", e / 60, e % 60), Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
        ]));
        right.push(Line::from(Span::styled("real Groth16 in the zkVM + Docker wrap", Style::default().fg(DIM))));
        if let Some(p) = &app.proof_progress {
            right.push(Line::from(Span::styled(trunc_n(p, 40), Style::default().fg(BLUE))));
        }
    } else if let Some(p) = &app.proof {
        right.push(Line::from(Span::styled("Proof ready", Style::default().fg(GREEN).add_modifier(Modifier::BOLD))));
        right.push(field("seal", &trunc(&p.seal)));
        right.push(field("journal", &trunc(&p.journal)));
        let ok = p.seal.starts_with("73c457ba");
        right.push(Line::from(Span::styled(
            if ok { "✓ selector 73c457ba (on-chain verifiable)" } else { "⚠ unexpected selector" },
            Style::default().fg(if ok { GREEN } else { RED }),
        )));
    } else {
        right.push(Line::from(Span::styled("Rules enforced in the zkVM:", Style::default().fg(DIM))));
        right.push(rule("amount = qty × unit_price", Some(amount.is_some())));
        right.push(rule("amount ≤ credit limit", amount.map(|a| a <= limit)));
        right.push(rule("ship date ≤ deadline", ship.map(|s| s <= deadline)));
        right.push(rule("currency = LC currency", Some(cur_match)));
        right.push(rule("seller ∈ approved set", Some(true)));
        right.push(rule("origin ∈ allowed set", Some(true)));
        right.push(rule("buyer balance ≥ amount", match (balance, amount) { (Some(b), Some(a)) => Some(b >= a), _ => None }));
        right.push(rule("issuer ed25519 signature", Some(true)));
    }
    right.push(Line::from(""));
    right.push(Line::from(vec![key("↑/↓"), Span::raw(" field  "), key("Enter"), Span::raw(" prove  "), key("^R"), Span::raw(" release")]));
    f.render_widget(Paragraph::new(right).block(panel("Seller · proof")).wrap(Wrap { trim: true }), cols[1]);
}

fn render_auditor(f: &mut Frame, app: &App, area: Rect) {
    // Profile selector header.
    let mut header: Vec<Span> = vec![Span::styled("Profile:  ", Style::default().fg(DIM))];
    for (i, p) in PROFILES.iter().enumerate() {
        let sel = i == app.audit_profile;
        header.push(Span::styled(
            format!(" {p} "),
            if sel { Style::default().fg(Color::Black).bg(GREEN).add_modifier(Modifier::BOLD) } else { Style::default().fg(DIM) },
        ));
        header.push(Span::raw(" "));
    }

    let mut lines = vec![
        Line::from(vec![Span::styled("Role: ", Style::default().fg(DIM)), Span::styled("AUDITOR", Style::default().fg(BLUE).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from("Selective disclosure: only the fields this profile is entitled"),
        Line::from("to see are revealed; the rest stay hidden but committed."),
        Line::from(""),
        Line::from(header),
        Line::from(""),
    ];
    if let Some(d) = &app.disclosure {
        lines.push(dfield("invoice amount", &d.amount));
        lines.push(dfield("quantity", &d.quantity));
        lines.push(dfield("unit price", &d.unit_price));
        lines.push(dfield("currency", &d.currency));
        lines.push(dfield("buyer_id", &d.buyer_id));
        lines.push(dfield("seller_id", &d.seller_id));
        lines.push(dfield("ship date", &d.ship_date));
        lines.push(dfield("origin id", &d.origin_id));
        lines.push(dfield("bol number", &d.bol_number));
        lines.push(dfield("carrier id", &d.carrier_id));
        lines.push(dfield("buyer balance", &d.balance));
        let m = d.commitment_match.to_lowercase();
        let (txt, c) = if m.starts_with("yes") { ("✓ matches on-chain commitment", GREEN) }
            else if m.starts_with("no") { ("✗ does NOT match on-chain commitment", RED) }
            else { ("commitment match: n/a", DIM) };
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(txt, Style::default().fg(c).add_modifier(Modifier::BOLD))));
    } else {
        lines.push(Line::from(Span::styled("Disclosure not opened yet.", Style::default().fg(DIM))));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![key("←/→"), Span::raw(" profile  "), key("a"), Span::raw(" Decrypt disclosure")]));
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
    let area = centered_rect(64, 80, f.area());
    f.render_widget(Clear, area);
    let row = |k: &'static str, d: &'static str| Line::from(vec![Span::styled(format!("  {k:<12}"), Style::default().fg(GREEN)), Span::styled(d, Style::default().fg(Color::White))]);
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("  Global", Style::default().fg(BLUE).add_modifier(Modifier::BOLD))),
        row("Tab", "switch role (Buyer / Seller / Auditor)"),
        row("?", "toggle this help     Esc/Ctrl+C  quit"),
        Line::from(""),
        Line::from(Span::styled("  Buyer", Style::default().fg(BLUE).add_modifier(Modifier::BOLD))),
        row("↑/↓", "select field (fund amount / balance)"),
        row("0-9", "edit the focused field"),
        row("f / x / s", "fund / refund / refresh status"),
        Line::from(""),
        Line::from(Span::styled("  Seller", Style::default().fg(BLUE).add_modifier(Modifier::BOLD))),
        row("↑/↓", "move between document fields"),
        row("type", "edit the focused field"),
        row("Enter", "generate real Groth16 proof"),
        row("Ctrl+R", "release escrow with the proof"),
        Line::from(""),
        Line::from(Span::styled("  Auditor", Style::default().fg(BLUE).add_modifier(Modifier::BOLD))),
        row("←/→", "switch disclosure profile"),
        row("a", "decrypt + verify commitment match"),
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

/// Disclosure field: dims + italicises hidden values so the withheld set is clear.
fn dfield(label: &str, value: &str) -> Line<'static> {
    let hidden = value.starts_with("hidden");
    let vstyle = if hidden {
        Style::default().fg(DIM).add_modifier(Modifier::ITALIC)
    } else {
        Style::default().fg(Color::White)
    };
    Line::from(vec![
        Span::styled(format!("  {label:<14}"), Style::default().fg(DIM)),
        Span::styled(trunc_n(value, 40), vstyle),
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
    trunc_n(s, 22)
}

fn trunc_n(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let head: String = s.chars().take(n).collect();
        format!("{head}…")
    }
}
