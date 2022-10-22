mod breakpoint;
mod state;

use std::fs::File;
use std::io;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use crate::{CircuitDescription, Config, Constraint, Preamble, Witness};

use breakpoint::Breakpoints;

pub use breakpoint::Breakpoint;
pub use state::State;

/// The Zk Debugger, it keeps track of breakpoints and the circuit description.
///
/// The Debugger maintains the encoded CDF file and breakpoints to provide
/// operations. The operations on the source code returns a
/// [`State`] which tells us where we are during debugging.
///
/// The Debugger is basically a [`CircuitDescription`] and breakpoints specified
/// by the user.
#[derive(Debug, Clone)]
pub struct ZkDebugger<S> {
    breakpoints: Breakpoints,
    cdf: CircuitDescription<S>,
    constraint: usize,
}

impl<S> Deref for ZkDebugger<S> {
    type Target = CircuitDescription<S>;

    fn deref(&self) -> &Self::Target {
        &self.cdf
    }
}

impl<S> DerefMut for ZkDebugger<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cdf
    }
}

impl<S> From<CircuitDescription<S>> for ZkDebugger<S> {
    fn from(cdf: CircuitDescription<S>) -> Self {
        Self {
            breakpoints: Breakpoints::default(),
            cdf,
            constraint: 0,
        }
    }
}

impl<S> ZkDebugger<S> {
    /// Configuration of the CDF file
    pub const fn config(&self) -> &Config {
        &self.cdf.preamble().config
    }

    /// Preamble for the CDF file
    pub const fn preamble(&self) -> &Preamble {
        self.cdf.preamble()
    }

    /// Add a breakpoint to the provided source/line.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{CircuitDescription, ZkDebugger, Breakpoint};
    ///
    /// let circuit = CircuitDescription::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from(circuit);
    /// let breakpoint = Breakpoint {
    ///     source: String::from("xyz"),
    ///     line: Some(40)   
    /// };
    ///
    /// debugger.add_breakpoint(String::from("xyz"), Some(40));
    /// assert_eq!(debugger.fetch_breakpoint(1), Some(&breakpoint));
    ///
    /// # Ok(()) }
    /// ```
    ///
    /// **Note**: If `line` is `None`, the breakpoint will be triggered in any
    /// incidence of `source`
    pub fn add_breakpoint(
        &mut self,
        source: String,
        line: Option<u64>,
    ) -> usize {
        self.breakpoints.add(source, line)
    }

    /// Remove a breakpoint with the provided id.
    ///
    /// If the id is not in the set, will return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{CircuitDescription, ZkDebugger, Breakpoint};
    ///
    /// let circuit = CircuitDescription::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from(circuit);
    /// let breakpoint = Breakpoint {
    ///     source: String::from("xyz"),
    ///     line: Some(40)   
    /// };
    ///
    /// debugger.add_breakpoint(String::from("xyz"), Some(40));
    /// assert_eq!(debugger.fetch_breakpoint(1), Some(&breakpoint));
    ///
    /// debugger.remove_breakpoint(1);
    /// assert_eq!(debugger.fetch_breakpoint(1), None);
    ///
    /// # Ok(()) }
    /// ```
    pub fn remove_breakpoint(&mut self, id: usize) -> Option<Breakpoint> {
        self.breakpoints.remove(id)
    }

    /// Fetch a breakpoint from an id returned from `add_breakpoint`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{CircuitDescription, ZkDebugger, Breakpoint};
    ///
    /// let circuit = CircuitDescription::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from(circuit);
    /// let breakpoint = Breakpoint {
    ///     source: String::from("xyz"),
    ///     line: Some(40)   
    /// };
    ///
    /// debugger.add_breakpoint(String::from("xyz"), Some(40));
    /// assert_eq!(debugger.fetch_breakpoint(1), Some(&breakpoint));
    ///
    /// # Ok(()) }
    /// ```
    pub fn fetch_breakpoint(&self, id: usize) -> Option<&Breakpoint> {
        self.breakpoints.find_breakpoint_from_id(id)
    }

    /// Underlying breakpoints repository
    pub const fn breakpoints(&self) -> &Breakpoints {
        &self.breakpoints
    }

    /// Remove all breakpoints that matches the source name
    pub fn clear_breakpoints(&mut self, source: &str) {
        self.breakpoints.clear(source);
    }
}

impl ZkDebugger<File> {
    /// Use a path to create a new circuit description. This uses
    /// [`CircuitDescription::from_reader`].
    pub fn open<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        CircuitDescription::open(path).map(Self::from)
    }
}

impl<S> ZkDebugger<S>
where
    S: io::Read + io::Seek,
{
    /// Create a CDF with the provided source and use it as backend for the
    /// debugger.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{CircuitDescription, ZkDebugger, Breakpoint};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    /// let breakpoint = Breakpoint {
    ///     source: String::from("xyz"),
    ///     line: Some(40)   
    /// };
    ///
    /// debugger.add_breakpoint(String::from("xyz"), Some(40));
    /// assert_eq!(debugger.fetch_breakpoint(1), Some(&breakpoint));
    ///
    /// # Ok(()) }
    /// ```
    pub fn from_reader(source: S) -> io::Result<Self> {
        CircuitDescription::from_reader(source).map(Self::from)
    }

    /// Attempt to fetch the current constraint from the source.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::ZkDebugger;
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    /// let constraint = debugger.fetch_current_constraint()?;
    ///
    /// assert_eq!(constraint.id(), 0);
    ///
    /// # Ok(()) }
    /// ```
    pub fn fetch_current_constraint(&mut self) -> io::Result<Constraint> {
        self.cdf.fetch_constraint(self.constraint)
    }

    /// Attempt to read an indexed constraint from the source.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::ZkDebugger;
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    /// let constraint = debugger.fetch_constraint(0)?;
    ///
    /// assert_eq!(constraint.id(), 0);
    ///
    /// # Ok(()) }
    /// ```
    pub fn fetch_constraint(&mut self, idx: usize) -> io::Result<Constraint> {
        self.cdf.fetch_constraint(idx)
    }

    /// Attempt to read an indexed witness from the source.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::ZkDebugger;
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    /// let witness = debugger.fetch_witness(0)?;
    ///
    /// assert_eq!(witness.id(), 0);
    ///
    /// # Ok(()) }
    /// ```
    pub fn fetch_witness(&mut self, idx: usize) -> io::Result<Witness> {
        self.cdf.fetch_witness(idx)
    }

    /// Move to previous source/line.
    ///
    /// May jump more than one constraint in case we have multiple constraints
    /// defined in a single source/file tuple.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{ZkDebugger, State};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    ///
    /// assert_eq!(debugger.afore()?, State::Beginning);
    ///
    /// # Ok(()) }
    /// ```
    pub fn afore(&mut self) -> io::Result<State> {
        let Self {
            breakpoints,
            cdf,
            constraint,
        } = self;

        let mut idx = *constraint;
        if idx == 0 {
            return Ok(State::Beginning);
        }

        let current = cdf.fetch_constraint(idx)?;
        let source = current.name().to_string();
        let line = current.line();

        loop {
            idx -= 1;

            if idx == 0 {
                *constraint = 0;
                return Ok(State::Beginning);
            }

            let current = cdf.fetch_constraint(idx)?;
            let is_invalid = !current.polynomial().evaluation;
            let different_line =
                source != current.name() || line != current.line();

            if different_line && is_invalid {
                *constraint = idx;
                return Ok(State::InvalidConstraint { id: idx });
            }

            if different_line {
                if let Some(id) = breakpoints.find_breakpoint(&current) {
                    *constraint = idx;
                    return Ok(State::Breakpoint { id });
                }
            }

            if different_line {
                break;
            }
        }

        *constraint = idx;
        Ok(State::Constraint { id: idx })
    }

    /// Continue the execution until EOF, breakpoint, or invalid constraint.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{ZkDebugger, State};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    ///
    /// assert_eq!(debugger.afore()?, State::Beginning);
    /// debugger.cont(); // continue execution
    ///
    /// # Ok(()) }
    /// ```
    pub fn cont(&mut self) -> io::Result<State> {
        let Self {
            breakpoints,
            cdf,
            constraint,
        } = self;

        let mut idx = *constraint;
        let eof = cdf.preamble().constraints.saturating_sub(1);

        if idx == eof {
            return Ok(State::End { id: idx });
        }

        let current = cdf.fetch_constraint(idx)?;
        let source = current.name().to_string();
        let line = current.line();

        loop {
            idx += 1;

            let current = cdf.fetch_constraint(idx)?;
            let is_invalid = !current.polynomial().evaluation;
            let different_line =
                source != current.name() || line != current.line();

            if different_line && is_invalid {
                *constraint = idx;
                return Ok(State::InvalidConstraint { id: idx });
            }

            if idx == eof {
                *constraint = idx;
                return Ok(State::End { id: idx });
            }

            if different_line {
                if let Some(id) = breakpoints.find_breakpoint(&current) {
                    *constraint = idx;
                    return Ok(State::Breakpoint { id });
                }
            }
        }
    }

    /// Attempt to jump to a given constraint.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{ZkDebugger, State};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    ///
    /// // goto 7 then go forward one step
    /// assert_eq!(debugger.goto(7)?, State::Constraint { id : 7 });
    /// assert_eq!(debugger.step()?, State::Constraint { id : 8 });
    ///
    /// # Ok(()) }
    /// ```
    pub fn goto(&mut self, idx: usize) -> io::Result<State> {
        let Self {
            cdf, constraint, ..
        } = self;

        if idx == 0 {
            *constraint = 0;
            return Ok(State::Beginning);
        }

        let current = cdf.fetch_constraint(idx)?;
        let is_invalid = !current.polynomial().evaluation;

        *constraint = idx;

        if is_invalid {
            return Ok(State::InvalidConstraint { id: idx });
        }

        if idx == cdf.preamble().constraints.saturating_sub(1) {
            return Ok(State::End { id: idx });
        }

        Ok(State::Constraint { id: idx })
    }

    /// Move to next source/line.
    ///
    /// May jump more than one constraint in case we have multiple constraints
    /// defined in a single source/file tuple.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{ZkDebugger, State};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    ///
    /// assert_eq!(debugger.step()?, State::Constraint { id : 6 });
    /// assert_eq!(debugger.step()?, State::Constraint { id : 7 });
    ///
    /// # Ok(()) }
    /// ```
    pub fn step(&mut self) -> io::Result<State> {
        let Self {
            breakpoints,
            cdf,
            constraint,
        } = self;

        let mut idx = *constraint;
        let eof = cdf.preamble().constraints.saturating_sub(1);

        if idx == eof {
            return Ok(State::End { id: idx });
        }

        let current = cdf.fetch_constraint(idx)?;
        let source = current.name().to_string();
        let line = current.line();

        loop {
            idx += 1;

            let current = cdf.fetch_constraint(idx)?;
            let is_invalid = !current.polynomial().evaluation;
            let different_line =
                source != current.name() || line != current.line();

            if different_line && is_invalid {
                *constraint = idx;
                return Ok(State::InvalidConstraint { id: idx });
            }

            if idx == eof {
                *constraint = idx;
                return Ok(State::End { id: idx });
            }

            if different_line {
                if let Some(id) = breakpoints.find_breakpoint(&current) {
                    *constraint = idx;
                    return Ok(State::Breakpoint { id });
                }
            }

            if different_line {
                break;
            }
        }

        *constraint = idx;
        Ok(State::Constraint { id: idx })
    }

    /// Reverse the execution until BOF, breakpoint, or invalid constraint.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// use dusk_cdf::{ZkDebugger, State, Breakpoint};
    /// use std::fs::File;
    ///
    /// let file = File::open("../assets/test.cdf")?;
    /// let mut debugger = ZkDebugger::from_reader(file)?;
    /// let breakpoint = Breakpoint {
    ///     source: String::from("xyz"),
    ///     line: Some(40)   
    /// };
    ///
    /// assert_eq!(debugger.turn()?, State::Beginning);
    ///
    /// # Ok(()) }
    /// ```
    pub fn turn(&mut self) -> io::Result<State> {
        let Self {
            breakpoints,
            cdf,
            constraint,
        } = self;

        let mut idx = *constraint;
        if idx == 0 {
            return Ok(State::Beginning);
        }

        let current = cdf.fetch_constraint(idx)?;
        let source = current.name().to_string();
        let line = current.line();

        loop {
            idx -= 1;

            if idx == 0 {
                *constraint = 0;
                return Ok(State::Beginning);
            }

            let current = cdf.fetch_constraint(idx)?;
            let is_invalid = !current.polynomial().evaluation;
            let different_line =
                source != current.name() || line != current.line();

            if different_line && is_invalid {
                *constraint = idx;
                return Ok(State::InvalidConstraint { id: idx });
            }

            if different_line {
                if let Some(id) = breakpoints.find_breakpoint(&current) {
                    *constraint = idx;
                    return Ok(State::Breakpoint { id });
                }
            }
        }
    }
}

#[test]
fn base_operations_wont_panic() -> io::Result<()> {
    let path = std::env!("CARGO_MANIFEST_DIR");
    let path = std::path::PathBuf::from(path)
        .parent()
        .expect("failed to updir")
        .join("assets")
        .join("test.cdf");

    let mut debugger = ZkDebugger::open(path)?;

    let b = debugger.add_breakpoint("rs".into(), Some(1));
    debugger.fetch_breakpoint(b).expect("breakpoint was added");
    debugger.remove_breakpoint(b).expect("breakpoint was added");
    debugger.clear_breakpoints("rs");

    debugger.fetch_current_constraint()?;
    debugger.fetch_constraint(0)?;
    debugger.fetch_witness(0)?;

    let state = debugger.cont()?;
    assert!(matches!(state, State::End { .. }));

    debugger.afore()?;
    debugger.goto(0)?;
    debugger.step()?;
    debugger.turn()?;

    Ok(())
}
