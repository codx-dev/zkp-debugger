mod config;
mod state;

#[cfg(test)]
mod tests;

use std::fs::File;
use std::io;
use std::path::Path;

use bat::line_range::{LineRange, LineRanges};
use bat::PrettyPrinter;
use crossterm::{cursor, queue, terminal};
use dusk_cdf::ZkDebugger;
use dusk_cdf::{BaseConfig, CircuitDescription, Constraint, Witness};
use prettytable::{cell, format, row, Table};

use crate::args::ParsedArgs;
use crate::commands::{Command, CommandParser};

pub use config::Config;
pub use state::State;

/// PDB App implementation
pub struct App<S> {
    config: Config,
    debugger: Option<ZkDebugger<S>>,
    parser: CommandParser,
}

impl App<File> {
    /// Open a CDF file
    pub fn open<P>(&mut self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        CircuitDescription::open(path).and_then(|cdf| self.set_cdf(cdf))
    }

    /// Load a new instance of the app
    pub fn load(args: ParsedArgs) -> io::Result<Self> {
        let config = Config::load()?;

        let mut app = Self {
            config,
            debugger: None,
            parser: CommandParser::default(),
        };

        let ParsedArgs { path } = args;

        if let Some(path) = path {
            app.open(path)?;
        }

        Ok(app)
    }

    /// Attempt to execute a given command
    pub fn execute(&mut self, command: Command) -> io::Result<State> {
        match command {
            Command::Afore => self.afore()?,

            Command::Breakpoint { source, line } => {
                self.add_breakpoint(source, line)?;
            }

            Command::Continue => self.cont()?,

            Command::Delete { id } => {
                self.delete_breakpoint(id);
            }

            Command::Empty => (),

            Command::Goto { id } => self.goto(id)?,

            Command::Help => self.help(),

            Command::Next => self.next()?,

            Command::Open { path } => self.open(path)?,

            Command::Print => self.print()?,

            Command::Quit => {
                println!("bye!");
                return Ok(State::ShouldQuit);
            }

            Command::Restart => self.goto(0)?,

            Command::Turn => self.turn()?,

            Command::Witness { id } => self.witness(id)?,
        }

        Ok(State::Continue)
    }

    /// Attempt to parse and execute a command
    pub fn parse_and_execute(&mut self, line: &str) -> io::Result<State> {
        self.parser.parse(line).and_then(|c| self.execute(c))
    }
}

impl<S> App<S>
where
    S: io::Read + io::Seek,
{
    fn fetch_current_constraint(
        &mut self,
    ) -> io::Result<(&Config, Constraint)> {
        let (config, debugger) = self.debugger_mut()?;
        let constraint = debugger.fetch_current_constraint()?;

        Ok((config, constraint))
    }

    fn render_current_constraint(&mut self) -> io::Result<()> {
        queue!(
            io::stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(1, 1)
        )?;

        let (config, constraint) = self.fetch_current_constraint()?;

        let name = constraint.name();
        let contents = constraint.contents();
        let line = constraint.line() as usize;

        Self::render(config, name, contents, line)
    }

    fn fetch_witness(&mut self, idx: usize) -> io::Result<(&Config, Witness)> {
        let (config, debugger) = self.debugger_mut()?;
        let witness = debugger.fetch_witness(idx)?;

        Ok((config, witness))
    }

    fn add_breakpoint(
        &mut self,
        source: String,
        line: Option<u64>,
    ) -> io::Result<()> {
        let (_, debugger) = self.debugger_mut()?;

        if !debugger.source_name_contains(&source) {
            println!("the provided name doesn't match to a constraint in the current cdf file!");

            return Ok(());
        }

        let id = debugger.add_breakpoint(source, line);

        println!("breakpoint added: #{}", id);

        Ok(())
    }

    fn print(&mut self) -> io::Result<()> {
        let (_, constraint) = self.fetch_current_constraint()?;

        let mut table = Self::table();

        table.set_titles(row!["item", "value"]);

        let selectors = constraint.polynomial().selectors;
        let evaluation = constraint.polynomial().evaluation;

        table.add_row(row!["Qm", hex::encode(selectors.qm)]);
        table.add_row(row!["Ql", hex::encode(selectors.ql)]);
        table.add_row(row!["Qr", hex::encode(selectors.qr)]);
        table.add_row(row!["Qd", hex::encode(selectors.qd)]);
        table.add_row(row!["Qc", hex::encode(selectors.qc)]);
        table.add_row(row!["Qo", hex::encode(selectors.qo)]);
        table.add_row(row!["PI", hex::encode(selectors.pi)]);
        table.add_row(row!["Qarith", hex::encode(selectors.qarith)]);
        table.add_row(row!["Qlogic", hex::encode(selectors.qlogic)]);
        table.add_row(row!["QRange", hex::encode(selectors.qrange)]);
        table.add_row(row!["QGVar", hex::encode(selectors.qgroup_variable)]);
        table.add_row(row!["QGFix", hex::encode(selectors.qfixed_add)]);

        table.printstd();

        let witnesses = constraint.polynomial().witnesses;

        let mut table = Self::table();

        table.set_titles(row!["id", "index", "value"]);

        let a = hex::encode(self.fetch_witness(witnesses.a)?.1.value());
        let b = hex::encode(self.fetch_witness(witnesses.b)?.1.value());
        let d = hex::encode(self.fetch_witness(witnesses.d)?.1.value());
        let o = hex::encode(self.fetch_witness(witnesses.o)?.1.value());

        table.add_row(row![format!("{}", witnesses.a), "a", a]);
        table.add_row(row![format!("{}", witnesses.b), "b", b]);
        table.add_row(row![format!("{}", witnesses.d), "d", d]);
        table.add_row(row![format!("{}", witnesses.o), "o", o]);

        table.printstd();

        println!("evaluation: {}", if evaluation { "ok" } else { "error" });

        Ok(())
    }

    fn witness(&mut self, id: usize) -> io::Result<()> {
        let (config, witness) = self.fetch_witness(id)?;

        Self::render_witness(config, witness)
    }

    fn set_cdf(&mut self, cdf: CircuitDescription<S>) -> io::Result<()> {
        self.debugger.replace(cdf.into());
        self.render_current_constraint()
    }

    fn goto(&mut self, idx: usize) -> io::Result<()> {
        self.debugger_mut()?.1.goto(idx)?;
        self.render_current_constraint()
    }

    fn next(&mut self) -> io::Result<()> {
        self.debugger_mut()?.1.step()?;
        self.render_current_constraint()
    }

    fn cont(&mut self) -> io::Result<()> {
        self.debugger_mut()?.1.cont()?;
        self.render_current_constraint()
    }

    fn afore(&mut self) -> io::Result<()> {
        self.debugger_mut()?.1.afore()?;
        self.render_current_constraint()
    }

    fn turn(&mut self) -> io::Result<()> {
        self.debugger_mut()?.1.turn()?;
        self.render_current_constraint()
    }
}

impl<S> App<S> {
    /// App configuration file
    pub const fn config(&self) -> &Config {
        &self.config
    }

    fn render(
        config: &Config,
        name: &str,
        contents: &str,
        line: usize,
    ) -> io::Result<()> {
        println!("{}", name);

        let margin = config.render.margin;
        let range = LineRanges::from(vec![LineRange::new(
            line.saturating_sub(margin),
            line.saturating_add(margin),
        )]);

        PrettyPrinter::new()
            .input_from_bytes(contents.as_bytes())
            .language("rust")
            .header(config.render.header)
            .grid(config.render.grid)
            .line_numbers(config.render.line_numbers)
            .line_ranges(range)
            .highlight(line)
            .theme(&config.render.theme)
            .print()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }

    fn render_witness(config: &Config, witness: Witness) -> io::Result<()> {
        let name = witness.name();
        let contents = witness.contents();
        let line = witness.line() as usize;

        Self::render(config, name, contents, line)
    }

    /// Underlying parser of the app
    pub fn parser(&self) -> &CommandParser {
        &self.parser
    }

    fn help(&self) {
        self.parser.instructions().iter().for_each(|i| {
            println!("{} - {}", i.syntax(), i.help());
        });
    }

    fn debugger<F>(&mut self, f: F)
    where
        F: FnMut(&mut ZkDebugger<S>),
    {
        self.debugger_mut()
            .map(|(_, d)| d)
            .map(f)
            .unwrap_or_else(|e| println!("{}", e))
    }

    fn debugger_mut(
        &mut self,
    ) -> io::Result<(&mut Config, &mut ZkDebugger<S>)> {
        let Self {
            debugger, config, ..
        } = self;

        debugger
            .as_mut()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "no circuit description is loaded. check command `open`",
                )
            })
            .map(|d| (config, d))
    }

    fn delete_breakpoint(&mut self, id: usize) {
        self.debugger(|d| {
            d.remove_breakpoint(id)
                .map(|b| println!("breakpoint #{} removed: {:?}", id, b))
                .unwrap_or_else(|| println!("breakpoint #{} not found", id))
        });
    }

    fn table() -> Table {
        let format = format::FormatBuilder::new()
            .column_separator('|')
            .borders('|')
            .separators(
                &[format::LinePosition::Top, format::LinePosition::Bottom],
                format::LineSeparator::new('-', '+', '+', '+'),
            )
            .padding(1, 1)
            .build();

        let mut table = Table::new();

        table.set_format(format);

        table
    }
}
