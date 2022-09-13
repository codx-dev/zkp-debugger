use std::io;
use std::path::PathBuf;

use crate::prelude::*;
use crossterm::{cursor, execute, terminal};

#[test]
fn app_base_commands_wont_panic() -> io::Result<()> {
    let mut stdout = io::stdout();

    execute!(stdout, terminal::EnterAlternateScreen, cursor::MoveTo(0, 0))?;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("failed to find root workspace dir")
        .join("assets")
        .join("test.cdf")
        .canonicalize()?;

    let args = ParsedArgs {
        path: Some(path.clone()),
    };
    let mut app = App::load(args)?;

    app.execute(Command::Empty)?;
    app.execute(Command::Next)?;
    app.execute(Command::Afore)?;
    app.execute(Command::Breakpoint {
        source: "hash".into(),
        line: Some(5),
    })?;
    app.execute(Command::Delete { id: 1 })?;
    app.execute(Command::Continue)?;
    app.execute(Command::Turn)?;
    app.execute(Command::Goto { id: 1 })?;
    app.execute(Command::Help)?;
    app.execute(Command::Print)?;
    app.execute(Command::Restart)?;
    app.execute(Command::Witness { id: 1 })?;
    app.execute(Command::Open { path })?;

    execute!(stdout, terminal::LeaveAlternateScreen)?;

    Ok(())
}
