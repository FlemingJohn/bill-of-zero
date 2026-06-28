//! Application state and the logic that drives it. Long-running work (proving,
//! on-chain calls) runs on worker threads that report back over an mpsc channel,
//! so the render loop stays responsive, animates a spinner, and streams proof
//! progress live.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::backend::{self, Disclosure, DocInput, Proof, TxResult};
use crate::config::Config;
use crate::util;

pub const TABS: [&str; 3] = ["Buyer", "Seller", "Auditor"];

/// Seller's trade-document fields (the seller no longer sets the buyer balance).
pub const SELLER_FIELDS: [&str; 7] = [
    "Quantity",
    "Unit price (USDC)",
    "Ship date (YYYY-MM-DD)",
    "Currency",
    "Country of origin",
    "Bill-of-lading no.",
    "Carrier / vessel",
];

/// Buyer's fields: how much to fund, and the buyer's own private balance.
pub const BUYER_FIELDS: [&str; 2] = ["Fund amount (USDC)", "Buyer balance (USDC)"];

/// Auditor entitlement profiles.
pub const PROFILES: [&str; 3] = ["tax", "regulator", "full"];

/// Live on-chain view of the escrow.
#[derive(Debug, Clone)]
pub struct Status {
    pub released: bool,
    pub escrow_bal: String,
    pub seller_bal: String,
}

/// An action awaiting y/n confirmation (it moves real money).
#[derive(Debug, Clone, Copy)]
pub enum Pending {
    Release,
    Refund,
}

impl Pending {
    pub fn prompt(&self) -> &'static str {
        match self {
            Pending::Release => "Release the escrow to the seller with this proof?",
            Pending::Refund => "Refund the escrow's remaining balance to the buyer?",
        }
    }
}

/// Results posted back from worker threads.
pub enum Msg {
    Proof(Result<Proof, String>),
    Progress(String),
    Tx { label: String, res: Result<TxResult, String> },
    Status(Result<Status, String>),
    Audit(Result<Disclosure, String>),
}

pub struct App {
    pub cfg: Config,
    pub source: String,
    pub tab: usize,
    pub seller_fields: [String; 7],
    pub seller_idx: usize,
    pub buyer_fields: [String; 2],
    pub buyer_idx: usize,
    pub audit_profile: usize,
    pub proof: Option<Proof>,
    pub proof_progress: Option<String>,
    pub disclosure: Option<Disclosure>,
    pub status: Option<Status>,
    pub busy: Option<String>,
    pub busy_since: Option<Instant>,
    pub spin: usize,
    pub confirm: Option<Pending>,
    pub show_help: bool,
    pub last_tx: Option<String>,
    pub log: Vec<String>,
    pub should_quit: bool,
    refreshing: bool,
    last_refresh: Option<Instant>,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
}

impl App {
    pub fn new(cfg: Config) -> Self {
        let (tx, rx) = channel();
        // The account that signs fund/release/refund. `deployer` holds the
        // tokens + XLM from the original deploy; override with BZ_SOURCE_KEY.
        let source = std::env::var("BZ_SOURCE_KEY").unwrap_or_else(|_| "deployer".to_string());
        let currency = cfg.terms.currency.clone();
        let mut app = Self {
            tab: 0,
            seller_fields: [
                "5000".into(),       // quantity
                "19".into(),         // unit price
                "2024-12-31".into(), // ship date
                currency,            // currency (defaults to the LC currency)
                "China".into(),      // country of origin
                "778412".into(),     // bol number
                "Maersk Line".into(),// carrier
            ],
            seller_idx: 0,
            buyer_fields: ["100000".into(), "150000".into()],
            buyer_idx: 0,
            audit_profile: 0,
            proof: None,
            proof_progress: None,
            disclosure: None,
            status: None,
            busy: None,
            busy_since: None,
            spin: 0,
            confirm: None,
            show_help: false,
            last_tx: None,
            log: vec!["Loaded LC terms + deployment. Tab switches role, ? for help.".into()],
            should_quit: false,
            refreshing: false,
            last_refresh: None,
            source,
            cfg,
            tx,
            rx,
        };
        app.refresh_status();
        app
    }

    pub fn spinner(&self) -> char {
        const FRAMES: [char; 4] = ['⠋', '⠙', '⠹', '⠸'];
        FRAMES[self.spin % FRAMES.len()]
    }

    pub fn elapsed_secs(&self) -> Option<u64> {
        self.busy_since.map(|t| t.elapsed().as_secs())
    }

    /// Derived invoice amount = quantity * unit_price (best-effort from inputs).
    pub fn derived_amount(&self) -> Option<u64> {
        let q: u64 = self.seller_fields[0].trim().parse().ok()?;
        let p: u64 = self.seller_fields[1].trim().parse().ok()?;
        Some(q.saturating_mul(p))
    }

    pub fn current_profile(&self) -> &'static str {
        PROFILES[self.audit_profile]
    }

    /// Called ~10x/sec: advance the spinner, auto-refresh, drain worker results.
    pub fn on_tick(&mut self) {
        self.spin = self.spin.wrapping_add(1);

        if self.busy.is_none() && !self.refreshing {
            let due = self.last_refresh.map_or(true, |t| t.elapsed().as_secs() >= 5);
            if due {
                self.refresh_status();
            }
        }

        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Msg::Progress(line) => self.proof_progress = Some(line),
                Msg::Status(Ok(s)) => {
                    self.refreshing = false;
                    self.status = Some(s);
                }
                Msg::Status(Err(e)) => {
                    self.refreshing = false;
                    self.push(format!("✗ status: {e}"));
                }
                Msg::Proof(Ok(p)) => {
                    self.clear_busy();
                    self.proof_progress = None;
                    self.push(format!("✓ real Groth16 proof — seal {}…", short(&p.seal)));
                    self.proof = Some(p);
                }
                Msg::Proof(Err(e)) => {
                    self.clear_busy();
                    self.proof_progress = None;
                    self.push(format!("✗ {e}"));
                }
                Msg::Tx { label, res: Ok(t) } => {
                    self.clear_busy();
                    match &t.hash {
                        Some(h) => {
                            self.last_tx = Some(h.clone());
                            self.push(format!("✓ {label} — tx {}…", &h[..h.len().min(12)]));
                        }
                        None => self.push(format!("✓ {label} submitted on-chain")),
                    }
                    self.last_refresh = None;
                }
                Msg::Tx { label, res: Err(e) } => {
                    self.clear_busy();
                    self.push(format!("✗ {label}: {e}"));
                }
                Msg::Audit(Ok(d)) => {
                    self.clear_busy();
                    self.push(format!("✓ disclosure decrypted ({} profile)", d.profile));
                    self.disclosure = Some(d);
                }
                Msg::Audit(Err(e)) => {
                    self.clear_busy();
                    self.push(format!("✗ audit: {e}"));
                }
            }
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if self.show_help {
            self.show_help = false;
            return;
        }
        if let Some(p) = self.confirm {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.confirm = None;
                    self.dispatch(p);
                }
                _ => {
                    self.confirm = None;
                    self.push("cancelled".into());
                }
            }
            return;
        }
        if key.code == KeyCode::Esc
            || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
        {
            self.should_quit = true;
            return;
        }
        match key.code {
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Tab => self.tab = (self.tab + 1) % TABS.len(),
            KeyCode::BackTab => self.tab = (self.tab + TABS.len() - 1) % TABS.len(),
            _ => match self.tab {
                0 => self.key_buyer(key),
                1 => self.key_seller(key),
                _ => self.key_auditor(key),
            },
        }
    }

    fn key_buyer(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.buyer_idx = (self.buyer_idx + BUYER_FIELDS.len() - 1) % BUYER_FIELDS.len(),
            KeyCode::Down => self.buyer_idx = (self.buyer_idx + 1) % BUYER_FIELDS.len(),
            KeyCode::Backspace => {
                self.buyer_fields[self.buyer_idx].pop();
            }
            KeyCode::Char('f') => self.do_fund(),
            KeyCode::Char('x') => self.confirm = Some(Pending::Refund),
            KeyCode::Char('s') => self.refresh_status(),
            KeyCode::Char(c) if c.is_ascii_digit() => self.buyer_fields[self.buyer_idx].push(c),
            _ => {}
        }
    }

    fn key_auditor(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('a') => self.do_audit(),
            KeyCode::Left => self.audit_profile = (self.audit_profile + PROFILES.len() - 1) % PROFILES.len(),
            KeyCode::Right | KeyCode::Char('t') => self.audit_profile = (self.audit_profile + 1) % PROFILES.len(),
            _ => {}
        }
    }

    /// Allowed input per seller field: numeric, date, or free text.
    fn seller_char_ok(idx: usize, c: char) -> bool {
        match idx {
            0 | 1 | 5 => c.is_ascii_digit(),                 // quantity, price, bol no.
            2 => c.is_ascii_digit() || c == '-',             // ship date
            _ => c.is_ascii_alphanumeric() || c == ' ' || c == '-', // currency, origin, carrier
        }
    }

    fn key_seller(&mut self, key: KeyEvent) {
        // Ctrl+R releases — a modifier so it never collides with typing 'r' into
        // a text field like "Carrier".
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('r') {
            self.request_release();
            return;
        }
        match key.code {
            KeyCode::Up => self.seller_idx = (self.seller_idx + SELLER_FIELDS.len() - 1) % SELLER_FIELDS.len(),
            KeyCode::Down => self.seller_idx = (self.seller_idx + 1) % SELLER_FIELDS.len(),
            KeyCode::Backspace => {
                self.seller_fields[self.seller_idx].pop();
            }
            KeyCode::Enter => self.do_prove(),
            KeyCode::Char(c) => {
                if Self::seller_char_ok(self.seller_idx, c) {
                    self.seller_fields[self.seller_idx].push(c);
                }
            }
            _ => {}
        }
    }

    fn dispatch(&mut self, p: Pending) {
        match p {
            Pending::Release => self.do_release(),
            Pending::Refund => self.do_refund(),
        }
    }

    fn request_release(&mut self) {
        if self.proof.is_none() {
            self.push("✗ no proof yet — generate one first".into());
            return;
        }
        self.confirm = Some(Pending::Release);
    }

    fn parse_input(&self) -> Result<DocInput, String> {
        let quantity = self.seller_fields[0].trim().parse::<u64>().map_err(|_| "quantity must be a whole number".to_string())?;
        let unit_price = self.seller_fields[1].trim().parse::<u64>().map_err(|_| "unit price must be a whole number".to_string())?;
        let ship_date = util::ymd_to_unix(&self.seller_fields[2]).map_err(|e| e.to_string())?;
        let currency = self.seller_fields[3].trim().to_string();
        let origin = self.seller_fields[4].trim().to_string();
        let bol_number = self.seller_fields[5].trim().parse::<u64>().map_err(|_| "bol number must be a whole number".to_string())?;
        let carrier = self.seller_fields[6].trim().to_string();
        let buyer_balance = self.buyer_fields[1].trim().parse::<u64>().map_err(|_| "buyer balance (Buyer tab) must be a whole number".to_string())?;
        Ok(DocInput { quantity, unit_price, ship_date_unix: ship_date, currency, origin, bol_number, carrier, buyer_balance_usdc: buyer_balance })
    }

    // --- actions ----------------------------------------------------------

    fn run<F>(&mut self, label: &str, f: F)
    where
        F: FnOnce() -> Msg + Send + 'static,
    {
        if let Some(b) = &self.busy {
            self.push(format!("⏳ busy ({b}) — wait for it to finish"));
            return;
        }
        self.busy = Some(label.to_string());
        self.busy_since = Some(Instant::now());
        let tx = self.tx.clone();
        thread::spawn(move || {
            let _ = tx.send(f());
        });
    }

    fn clear_busy(&mut self) {
        self.busy = None;
        self.busy_since = None;
    }

    pub fn refresh_status(&mut self) {
        if self.refreshing {
            return;
        }
        self.refreshing = true;
        self.last_refresh = Some(Instant::now());
        let cfg = self.cfg.clone();
        let src = self.source.clone();
        let tx = self.tx.clone();
        thread::spawn(move || {
            let res = (|| {
                Ok::<Status, String>(Status {
                    released: backend::is_released(&cfg, &src).map_err(|e| e.to_string())?,
                    escrow_bal: backend::balance(&cfg, &src, &cfg.deployment.escrow).map_err(|e| e.to_string())?,
                    seller_bal: backend::balance(&cfg, &src, &cfg.deployment.seller).map_err(|e| e.to_string())?,
                })
            })();
            let _ = tx.send(Msg::Status(res));
        });
    }

    fn do_prove(&mut self) {
        let input = match self.parse_input() {
            Ok(i) => i,
            Err(e) => {
                self.push(format!("✗ {e}"));
                return;
            }
        };
        if let Some(b) = &self.busy {
            self.push(format!("⏳ busy ({b})"));
            return;
        }
        self.busy = Some("prove".to_string());
        self.busy_since = Some(Instant::now());
        self.proof_progress = Some("launching zkVM…".into());
        let cfg = self.cfg.clone();
        let tx = self.tx.clone();
        thread::spawn(move || {
            let txp = tx.clone();
            let progress = move |line: String| {
                let _ = txp.send(Msg::Progress(line));
            };
            let res = backend::prove(&cfg, &input, &progress).map_err(|e| e.to_string());
            let _ = tx.send(Msg::Proof(res));
        });
    }

    fn do_fund(&mut self) {
        let amount = match self.buyer_fields[0].trim().parse::<u64>() {
            Ok(a) if a > 0 => a,
            _ => {
                self.push("✗ fund amount must be a positive number".into());
                return;
            }
        };
        let cfg = self.cfg.clone();
        let src = self.source.clone();
        self.run("fund", move || Msg::Tx {
            label: "fund".into(),
            res: backend::fund(&cfg, &src, amount).map_err(|e| e.to_string()),
        });
    }

    fn do_release(&mut self) {
        let Some(p) = self.proof.clone() else {
            self.push("✗ no proof yet — generate one on the Seller tab".into());
            return;
        };
        let cfg = self.cfg.clone();
        let src = self.source.clone();
        self.run("release", move || Msg::Tx {
            label: "release".into(),
            res: backend::release(&cfg, &src, &p.seal, &p.journal).map_err(|e| e.to_string()),
        });
    }

    fn do_refund(&mut self) {
        let cfg = self.cfg.clone();
        let src = self.source.clone();
        self.run("refund", move || Msg::Tx {
            label: "refund".into(),
            res: backend::refund(&cfg, &src).map_err(|e| e.to_string()),
        });
    }

    fn do_audit(&mut self) {
        let cfg = self.cfg.clone();
        let src = self.source.clone();
        let profile = self.current_profile().to_string();
        self.run("audit", move || {
            // Fetch the on-chain commitment so the auditor can verify the match.
            let expected = backend::escrow_disclosure(&cfg, &src);
            Msg::Audit(backend::audit(&cfg, &profile, expected.as_deref()).map_err(|e| e.to_string()))
        });
    }

    fn push(&mut self, line: String) {
        self.log.push(line);
        let n = self.log.len();
        if n > 6 {
            self.log.drain(0..n - 6);
        }
    }
}

fn short(s: &str) -> String {
    s.chars().take(12).collect()
}
