//! Application state and the logic that drives it. Long-running work (proving,
//! on-chain calls) runs on worker threads that report back over an mpsc channel,
//! so the render loop stays responsive and can animate a spinner.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::backend::{self, Disclosure, DocInput, Proof};
use crate::config::Config;
use crate::util;

pub const TABS: [&str; 3] = ["Buyer", "Seller", "Auditor"];
pub const FIELDS: [&str; 3] = ["Invoice amount (USDC)", "Ship date (YYYY-MM-DD)", "Buyer balance (USDC)"];

/// Live on-chain view of the escrow.
#[derive(Debug, Clone)]
pub struct Status {
    pub released: bool,
    pub escrow_bal: String,
    pub seller_bal: String,
}

/// Results posted back from worker threads.
pub enum Msg {
    Proof(Result<Proof, String>),
    Tx { label: String, res: Result<String, String> },
    Status(Result<Status, String>),
    Audit(Result<Disclosure, String>),
}

pub struct App {
    pub cfg: Config,
    pub source: String,
    pub tab: usize,
    pub fields: [String; 3],
    pub field_idx: usize,
    pub dev_mode: bool,
    pub proof: Option<Proof>,
    pub disclosure: Option<Disclosure>,
    pub status: Option<Status>,
    pub busy: Option<String>,
    pub spin: usize,
    pub log: Vec<String>,
    pub should_quit: bool,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
}

impl App {
    pub fn new(cfg: Config) -> Self {
        let (tx, rx) = channel();
        // The account that signs fund/release. `deployer` holds the tokens + XLM
        // from the original deploy; override with BZ_SOURCE_KEY if needed.
        let source = std::env::var("BZ_SOURCE_KEY").unwrap_or_else(|_| "deployer".to_string());
        let mut app = Self {
            tab: 0,
            fields: ["95000".into(), "2024-12-31".into(), "150000".into()],
            field_idx: 0,
            dev_mode: true,
            proof: None,
            disclosure: None,
            status: None,
            busy: None,
            spin: 0,
            log: vec!["Loaded LC terms + deployment. Press Tab to switch role.".into()],
            should_quit: false,
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

    /// Called ~10x/sec: advance the spinner and drain worker results.
    pub fn on_tick(&mut self) {
        self.spin = self.spin.wrapping_add(1);
        while let Ok(msg) = self.rx.try_recv() {
            self.busy = None;
            match msg {
                Msg::Proof(Ok(p)) => {
                    let kind = if p.dev_mode { "dev" } else { "real" };
                    self.push(format!("✓ proof ({kind}) — seal {}…", short(&p.seal)));
                    self.proof = Some(p);
                }
                Msg::Proof(Err(e)) => self.push(format!("✗ {e}")),
                Msg::Tx { label, res: Ok(_) } => {
                    self.push(format!("✓ {label} submitted on-chain"));
                    self.refresh_status();
                }
                Msg::Tx { label, res: Err(e) } => self.push(format!("✗ {label}: {e}")),
                Msg::Status(Ok(s)) => self.status = Some(s),
                Msg::Status(Err(e)) => self.push(format!("✗ status: {e}")),
                Msg::Audit(Ok(d)) => {
                    self.push("✓ disclosure decrypted by auditor".into());
                    self.disclosure = Some(d);
                }
                Msg::Audit(Err(e)) => self.push(format!("✗ audit: {e}")),
            }
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        // Global quit.
        if key.code == KeyCode::Esc
            || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
        {
            self.should_quit = true;
            return;
        }
        match key.code {
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
            KeyCode::Char('f') => self.do_fund(),
            KeyCode::Char('s') => self.refresh_status(),
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    fn key_auditor(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('a') => self.do_audit(),
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    fn key_seller(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.field_idx = (self.field_idx + FIELDS.len() - 1) % FIELDS.len(),
            KeyCode::Down => self.field_idx = (self.field_idx + 1) % FIELDS.len(),
            KeyCode::Backspace => {
                self.fields[self.field_idx].pop();
            }
            KeyCode::Enter | KeyCode::Char('p') => self.do_prove(),
            KeyCode::Char('d') => self.dev_mode = !self.dev_mode,
            KeyCode::Char('r') => self.do_release(),
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char(c) => {
                // Digits everywhere; '-' only in the date field.
                let date_field = self.field_idx == 1;
                if c.is_ascii_digit() || (c == '-' && date_field) {
                    self.fields[self.field_idx].push(c);
                }
            }
            _ => {}
        }
    }

    fn parse_input(&self) -> Result<DocInput, String> {
        let amount = self.fields[0].trim().parse::<u64>().map_err(|_| "invoice amount must be a whole number".to_string())?;
        let ship = util::ymd_to_unix(&self.fields[1]).map_err(|e| e.to_string())?;
        let balance = self.fields[2].trim().parse::<u64>().map_err(|_| "buyer balance must be a whole number".to_string())?;
        Ok(DocInput { amount_usdc: amount, ship_date_unix: ship, buyer_balance_usdc: balance })
    }

    // --- actions (each spawns a worker thread) ----------------------------

    fn run<F>(&mut self, label: &str, f: F)
    where
        F: FnOnce() -> Msg + Send + 'static,
    {
        if let Some(b) = &self.busy {
            self.push(format!("⏳ busy ({b}) — wait for it to finish"));
            return;
        }
        self.busy = Some(label.to_string());
        let tx = self.tx.clone();
        thread::spawn(move || {
            let _ = tx.send(f());
        });
    }

    pub fn refresh_status(&mut self) {
        let cfg = self.cfg.clone();
        let src = self.source.clone();
        self.run("status", move || {
            let res = (|| {
                Ok::<Status, String>(Status {
                    released: backend::is_released(&cfg, &src).map_err(|e| e.to_string())?,
                    escrow_bal: backend::balance(&cfg, &src, &cfg.deployment.escrow).map_err(|e| e.to_string())?,
                    seller_bal: backend::balance(&cfg, &src, &cfg.deployment.seller).map_err(|e| e.to_string())?,
                })
            })();
            Msg::Status(res)
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
        let cfg = self.cfg.clone();
        let dev = self.dev_mode;
        self.run("prove", move || {
            Msg::Proof(backend::prove(&cfg, &input, dev).map_err(|e| e.to_string()))
        });
    }

    fn do_fund(&mut self) {
        let cfg = self.cfg.clone();
        let src = self.source.clone();
        self.run("fund", move || Msg::Tx {
            label: "fund".into(),
            res: backend::fund(&cfg, &src, 100_000).map_err(|e| e.to_string()),
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

    fn do_audit(&mut self) {
        let cfg = self.cfg.clone();
        self.run("audit", move || {
            Msg::Audit(backend::audit(&cfg).map_err(|e| e.to_string()))
        });
    }

    fn push(&mut self, line: String) {
        self.log.push(line);
        let n = self.log.len();
        if n > 8 {
            self.log.drain(0..n - 8);
        }
    }
}

fn short(s: &str) -> String {
    s.chars().take(12).collect()
}
