#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;

use bat::line_range::LineRanges;
use bat::PrettyPrinter;
use crossterm::{cursor, queue, terminal};
use dusk_plonk_cdf::{CircuitDescription, CircuitDescriptionFile, Constraint, Preamble, Source};
use prettytable::{cell, format, row, Table};

use super::{Command, CommandParser};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Breakpoint {
    source: String,
    line: Option<u64>,
}

/// App configuration
#[derive(Debug, Clone)]
pub struct Config {
    source_render_margin: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            source_render_margin: 10,
        }
    }
}

/// PDB App implementation
pub struct App<S> {
    config: Config,
    parser: CommandParser,
    breakpoints: HashMap<u64, Breakpoint>,
    next_breakpoint: u64,
    cdf: Option<CircuitDescription<S>>,
    constraint: Option<Constraint>,
    preamble: Preamble,
    is_last_constraint_ok: bool,
    last_constraint: usize,
    finished: bool,
}

impl<S> Default for App<S> {
    fn default() -> Self {
        Self {
            config: Config::default(),
            parser: CommandParser::default(),
            breakpoints: HashMap::default(),
            next_breakpoint: u64::default(),
            cdf: None,
            constraint: None,
            preamble: Preamble::default(),
            is_last_constraint_ok: bool::default(),
            last_constraint: usize::default(),
            finished: bool::default(),
        }
    }
}

impl App<File> {
    /// Open a CDF file
    pub fn open_path<P>(&mut self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        CircuitDescriptionFile::open_read(path)
            .and_then(|cdf| self.set_cdf(cdf))
            .and_then(|_| self.restart())
    }

    /// Attempt to execute a given command
    pub fn execute(&mut self, command: Command) -> io::Result<State> {
        match command {
            Command::Afore => self.afore().and_then(|_| self.render())?,

            Command::Breakpoint { source, line } => {
                self.add_breakpoint(source, line);
            }

            Command::Continue => self.cont().and_then(|_| self.render())?,

            Command::Delete { id } => {
                self.delete_breakpoint(id);
            }

            Command::Help => self.help(),

            Command::Empty => (),

            Command::Goto { id } => self.goto(id).and_then(|_| self.render())?,

            Command::Next => self.next().and_then(|_| self.render())?,

            Command::Open { path } => self.open_path(path).and_then(|_| self.render())?,

            Command::Print => self.print()?,

            Command::Restart => self.restart().and_then(|_| self.render())?,

            Command::Turn => self.turn().and_then(|_| self.render())?,

            Command::Quit => {
                println!("bye!");
                return Ok(State::ShouldQuit);
            }

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
    fn set_cdf(&mut self, cdf: CircuitDescription<S>) -> io::Result<()> {
        self.preamble = *cdf.preamble();
        self.cdf.replace(cdf);

        self.restart()
    }

    fn jump(&mut self, constraint: usize) -> io::Result<()> {
        let constraint = self
            .cdf_mut()
            .and_then(|cdf| cdf.fetch_constraint(constraint))?;

        self.last_constraint = constraint.id() as usize;
        self.is_last_constraint_ok = constraint.is_ok();
        self.finished =
            self.last_constraint == self.preamble.constraints().saturating_sub(1) as usize;
        self.constraint.replace(constraint);

        Ok(())
    }

    fn goto(&mut self, id: u64) -> io::Result<()> {
        self.jump(id as usize)
    }

    fn jump_one(&mut self) -> io::Result<()> {
        self.jump(self.last_constraint.saturating_add(1))
    }

    fn cont(&mut self) -> io::Result<()> {
        loop {
            self.jump_one()?;

            if self.should_interrupt() {
                return Ok(());
            }
        }
    }

    fn jump_previous(&mut self) -> io::Result<()> {
        self.jump(self.last_constraint.saturating_sub(1))
    }

    fn afore(&mut self) -> io::Result<()> {
        self.jump_previous()
    }

    fn next(&mut self) -> io::Result<()> {
        self.jump(self.last_constraint.saturating_add(1))
    }

    fn restart(&mut self) -> io::Result<()> {
        self.constraint.take();
        self.finished = false;
        self.last_constraint = 0;
        self.is_last_constraint_ok = true;

        self.jump(0)
    }

    fn turn(&mut self) -> io::Result<()> {
        loop {
            self.jump_previous()?;

            if self.last_constraint == 0 || self.should_interrupt() {
                return Ok(());
            }
        }
    }

    fn witness(&mut self, id: u64) -> io::Result<()> {
        let witness = self.cdf_mut()?.fetch_witness(id as usize)?;

        let mut table = Self::table();

        table.set_titles(row!["id", "value"]);
        table.add_row(row![
            format!("{}", witness.id()),
            hex::encode(witness.value())
        ]);
        table.printstd();

        self.render_source(witness.source())?;

        Ok(())
    }
}

impl<S> App<S> {
    fn add_breakpoint(&mut self, source: String, line: Option<u64>) -> u64 {
        let id = self.next_breakpoint;

        self.breakpoints.insert(id, Breakpoint { source, line });

        println!("breakpoint added: #{}", self.next_breakpoint);

        self.next_breakpoint += 1;

        id
    }

    fn cdf_mut(&mut self) -> io::Result<&mut CircuitDescription<S>> {
        self.cdf
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to load CDF file"))
    }

    fn constraint(&self) -> io::Result<&Constraint> {
        self.constraint
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to fetch constraint file"))
    }

    fn delete_breakpoint(&mut self, id: u64) {
        match self.breakpoints.remove(&id) {
            Some(_) => println!("breakpoint #{} removed", id),
            None => println!("breakpoint #{} not found", id),
        }
    }

    fn help(&self) {
        self.parser.instructions().iter().for_each(|i| {
            println!("{} - {}", i.syntax(), i.help());
        });
    }

    fn print(&self) -> io::Result<()> {
        self.render()?;

        let (qm, ql, qr, qd, qc, qo, pi, a, b, d, o, _re) =
            self.constraint()?.polynomial().internals();

        let mut table = Self::table();

        table.set_titles(row!["selector", "value"]);

        table.add_row(row!["Qm", hex::encode(qm)]);
        table.add_row(row!["Ql", hex::encode(ql)]);
        table.add_row(row!["Qr", hex::encode(qr)]);
        table.add_row(row!["Qd", hex::encode(qd)]);
        table.add_row(row!["Qc", hex::encode(qc)]);
        table.add_row(row!["Qo", hex::encode(qo)]);
        table.add_row(row!["pi", hex::encode(pi)]);

        table.printstd();

        let mut table = Self::table();

        table.set_titles(row!["id", "index", "value"]);

        table.add_row(row![format!("{}", a.index()), "a", hex::encode(a.value())]);
        table.add_row(row![format!("{}", b.index()), "b", hex::encode(b.value())]);
        table.add_row(row![format!("{}", d.index()), "d", hex::encode(d.value())]);
        table.add_row(row![format!("{}", o.index()), "o", hex::encode(o.value())]);

        table.printstd();

        Ok(())
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

    fn is_breakpoint(&self) -> bool {
        self.constraint
            .as_ref()
            .map(|constraint| constraint.source())
            .map(|source| {
                self.breakpoints.values().any(|b| {
                    source.path().contains(&b.source)
                        && b.line.map(|line| line == source.line()).unwrap_or(true)
                })
            })
            .unwrap_or(false)
    }

    /// Underlying parser of the app
    pub fn parser(&self) -> &CommandParser {
        &self.parser
    }

    fn render(&self) -> io::Result<()> {
        self.constraint
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "undefined constraint"))
            .and_then(|c| {
                queue!(
                    io::stdout(),
                    terminal::Clear(terminal::ClearType::All),
                    cursor::MoveTo(1, 1)
                )?;

                self.render_source(c.source())?;

                println!("evaluation: {}", c.is_ok());

                if self.finished {
                    println!("execution finished");
                }

                Ok(())
            })
    }

    fn render_source(&self, source: &Source) -> io::Result<()> {
        let path = source.canonical_path()?;

        println!("{}", path.display());

        let margin = self.config.source_render_margin;

        let line = source.line() as usize;
        let range = LineRanges::from(vec![bat::line_range::LineRange::new(
            line.saturating_sub(margin),
            line.saturating_add(margin),
        )]);

        PrettyPrinter::new()
            .input_file(path)
            .language("rust")
            .header(true)
            .grid(true)
            .line_numbers(true)
            .line_ranges(range)
            .highlight(line)
            .print()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }

    fn should_interrupt(&self) -> bool {
        self.finished || !self.is_last_constraint_ok || self.is_breakpoint()
    }
}

/// Resulting state of an executed command
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    /// Application should continue
    Continue,
    /// Application should quit
    ShouldQuit,
}

impl State {
    /// Check if the application should continue executing, given a state
    pub const fn should_continue(&self) -> bool {
        matches!(self, Self::Continue)
    }
}
