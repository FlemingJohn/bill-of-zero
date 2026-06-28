//! Bill of Zero — terminal UI.
//!
//! A keyboard-driven front end for the ZK Letter-of-Credit settlement demo.
//! It shells out to the existing `host` binary (proving + auditor disclosure)
//! and the `stellar` CLI (on-chain fund/release), so there is no server, no
//! browser, and no wallet extension to keep alive.
//!
//! Run from inside the repo:  cargo run -p bz-tui
//! Signing key:  set BZ_SOURCE_KEY to a `stellar keys` identity (default "deployer").

mod app;
mod backend;
mod config;
mod ui;
mod util;

use std::time::Duration;

use anyhow::Result;
use ratatui::crossterm::event::{self, Event};

use app::App;
use config::Config;

fn main() -> Result<()> {
    let cfg = Config::load()?;
    let mut app = App::new(cfg);

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key);
            }
        }
        app.on_tick();

        if app.should_quit {
            return Ok(());
        }
    }
}
