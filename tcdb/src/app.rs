mod breakpoint;
mod config;
mod state;

use std::fs::File;
use std::io;
use std::path::Path;

use bat::line_range::{LineRange, LineRanges};
use bat::PrettyPrinter;
use crossterm::{cursor, queue, terminal};
use dusk_cdf::{BaseConfig, CircuitDescription, Constraint, Witness};
use prettytable::{cell, format, row, Table};

use crate::{Command, CommandParser, ParsedArgs};

pub use config::Config;
pub use state::State;

use breakpoint::Breakpoints;

/// PDB App implementation
pub struct App<S> {
    config: Config,
    cdf: Option<CircuitDescription<S>>,
    constraint: Option<usize>,
    parser: CommandParser,
    breakpoints: Breakpoints,
}

impl App<File> {
    /// Load a new instance of the app
    pub fn load(args: ParsedArgs) -> io::Result<Self> {
        let config = Config::load()?;

        let mut app = Self {
            config,
            cdf: None,
            constraint: None,
            parser: CommandParser::default(),
            breakpoints: Breakpoints::default(),
        };

        let ParsedArgs { path } = args;

        if let Some(path) = path {
            app.open(path)?;
        }

        Ok(app)
    }

    /// Open a CDF file
    pub fn open<P>(&mut self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        CircuitDescription::open(path).and_then(|cdf| self.set_cdf(cdf))
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
    fn cdf(&self) -> io::Result<&CircuitDescription<S>> {
        self.cdf
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "CDF file not loaded!"))
    }

    fn cdf_mut(&mut self) -> io::Result<(&Config, &Breakpoints, &mut CircuitDescription<S>)> {
        let Self {
            config,
            cdf,
            breakpoints,
            ..
        } = self;

        let cdf = cdf
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "CDF file not loaded!"))?;

        Ok((config, breakpoints, cdf))
    }

    fn fetch_constraint(&mut self, idx: usize) -> io::Result<(&Config, &Breakpoints, Constraint)> {
        let (config, breakpoints, cdf) = self.cdf_mut()?;
        let constraint = cdf.fetch_constraint(idx)?;

        Ok((config, breakpoints, constraint))
    }

    fn fetch_witness(&mut self, idx: usize) -> io::Result<(&Config, Witness)> {
        let (config, _, cdf) = self.cdf_mut()?;
        let witness = cdf.fetch_witness(idx)?;

        Ok((config, witness))
    }

    fn goto(&mut self, idx: usize) -> io::Result<()> {
        let (config, _, constraint) = self.fetch_constraint(idx)?;

        Self::render_constraint(config, constraint)?;

        self.constraint.replace(idx);

        Ok(())
    }

    fn set_cdf(&mut self, cdf: CircuitDescription<S>) -> io::Result<()> {
        self.constraint.take();
        self.cdf.replace(cdf);
        self.goto(0)
    }

    fn add_breakpoint(&mut self, source: String, line: Option<u64>) -> io::Result<()> {
        if !self.cdf()?.source_name_contains(&source) {
            println!("the provided name doesn't match to a constraint in the current cdf file!");

            return Ok(());
        }

        let id = self.breakpoints.add(source, line);

        println!("breakpoint added: #{}", id);

        Ok(())
    }

    fn print(&mut self) -> io::Result<()> {
        let idx = match self.constraint {
            Some(idx) => idx,
            None => {
                println!("no constraint loaded!");
                return Ok(());
            }
        };

        let mut table = Self::table();

        table.set_titles(row!["item", "value"]);

        let (_, _, constraint) = self.fetch_constraint(idx)?;

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

    fn next(&mut self) -> io::Result<()> {
        let mut idx = match self.constraint {
            Some(idx) => idx,
            None => {
                println!("no constraint loaded!");
                return Ok(());
            }
        };

        let constraints = self.cdf()?.preamble().constraints;
        if idx >= constraints.saturating_sub(1) {
            println!("execution finished");
            return Ok(());
        }

        let (_, _, constraint) = self.fetch_constraint(idx)?;

        let mut name = constraint.name().to_owned();
        let mut line = constraint.line();

        loop {
            if idx >= constraints.saturating_sub(1) {
                println!("execution finished");
                idx = constraints.saturating_sub(1);
                break;
            }

            let (_, _, constraint) = self.fetch_constraint(idx)?;

            if name != constraint.name()
                || line != constraint.line()
                || !constraint.polynomial().evaluation
            {
                break;
            }

            idx += 1;
            name = constraint.name().to_owned();
            line = constraint.line();
        }

        self.goto(idx)
    }

    fn cont(&mut self) -> io::Result<()> {
        let mut idx = match self.constraint {
            Some(idx) => idx,
            None => {
                println!("no constraint loaded!");
                return Ok(());
            }
        };

        let constraints = self.cdf()?.preamble().constraints;
        if idx >= constraints.saturating_sub(1) {
            println!("execution finished");
            return Ok(());
        }

        loop {
            if idx >= constraints.saturating_sub(1) {
                println!("execution finished");
                idx = constraints.saturating_sub(1);
                break;
            }

            let (_, breakpoints, constraint) = self.fetch_constraint(idx)?;

            if !constraint.polynomial().evaluation
                || breakpoints.is_breakpoint(constraint.name(), constraint.line())
            {
                break;
            }

            idx += 1;
        }

        self.goto(idx)
    }

    fn afore(&mut self) -> io::Result<()> {
        let mut idx = match self.constraint {
            Some(idx) => idx,
            None => {
                println!("no constraint loaded!");
                return Ok(());
            }
        };

        if idx == 0 {
            println!("beginning of file");
            return Ok(());
        }

        let (_, _, constraint) = self.fetch_constraint(idx)?;

        let mut name = constraint.name().to_owned();
        let mut line = constraint.line();

        loop {
            if idx == 0 {
                println!("beginning of file");
                break;
            }

            let (_, _, constraint) = self.fetch_constraint(idx)?;

            if name != constraint.name()
                || line != constraint.line()
                || !constraint.polynomial().evaluation
            {
                break;
            }

            idx = idx.saturating_sub(1);
            name = constraint.name().to_owned();
            line = constraint.line();
        }

        self.goto(idx)
    }

    fn turn(&mut self) -> io::Result<()> {
        let mut idx = match self.constraint {
            Some(idx) => idx,
            None => {
                println!("no constraint loaded!");
                return Ok(());
            }
        };

        loop {
            if idx == 0 {
                println!("beginning of file");
                break;
            }

            let (_, breakpoints, constraint) = self.fetch_constraint(idx)?;

            if !constraint.polynomial().evaluation
                || breakpoints.is_breakpoint(constraint.name(), constraint.line())
            {
                break;
            }

            idx = idx.saturating_sub(1);
        }

        self.goto(idx)
    }
}

impl<S> App<S> {
    /// App configuration file
    pub const fn config(&self) -> &Config {
        &self.config
    }

    fn render(config: &Config, name: &str, contents: &str, line: usize) -> io::Result<()> {
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

    fn render_constraint(config: &Config, constraint: Constraint) -> io::Result<()> {
        queue!(
            io::stdout(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(1, 1)
        )?;

        let name = constraint.name();
        let contents = constraint.contents();
        let line = constraint.line() as usize;

        Self::render(config, name, contents, line)
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

    fn delete_breakpoint(&mut self, id: usize) {
        match self.breakpoints.remove(id) {
            Some(b) => println!("breakpoint #{} removed: {:?}", id, b),
            None => println!("breakpoint #{} not found", id),
        }
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
