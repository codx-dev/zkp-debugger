mod breakpoint;
mod state;

use std::io;
use std::ops::{Deref, DerefMut};

use crate::{CircuitDescription, Config, Constraint, Preamble, Witness};

use breakpoint::Breakpoints;

pub use breakpoint::Breakpoint;
pub use state::State;

/// ZKP Debugger with CDF backend
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

    /// Add a breakpoint to the provided source/line
    ///
    /// If `line` is `None`, the breakpoint will be triggered in any incidence of `source`
    pub fn add_breakpoint(&mut self, source: String, line: Option<u64>) -> usize {
        self.breakpoints.add(source, line)
    }

    /// Remove a breakpoint with the provided id.
    ///
    /// If the id is not in the set, will return `None`.
    pub fn remove_breakpoint(&mut self, id: usize) -> Option<Breakpoint> {
        self.breakpoints.remove(id)
    }

    /// Fetch a breakpoint from an id returned from `add_breakpoint`.
    pub fn fetch_breakpoint(&mut self, id: usize) -> Option<&Breakpoint> {
        self.breakpoints.find_breakpoint_from_id(id)
    }
}

impl<S> ZkDebugger<S>
where
    S: io::Read + io::Seek,
{
    /// Create a CDF with the provided source and use it as backend for the debugger
    pub fn from_reader(source: S) -> io::Result<Self> {
        CircuitDescription::from_reader(source).map(Self::from)
    }

    /// Attempt to fetch the current constraint from the source
    pub fn fetch_current_constraint(&mut self) -> io::Result<Constraint> {
        self.cdf.fetch_constraint(self.constraint)
    }

    /// Attempt to read an indexed constraint from the source
    pub fn fetch_constraint(&mut self, idx: usize) -> io::Result<Constraint> {
        self.cdf.fetch_constraint(idx)
    }

    /// Attempt to read an indexed witness from the source
    pub fn fetch_witness(&mut self, idx: usize) -> io::Result<Witness> {
        self.cdf.fetch_witness(idx)
    }

    /// Move to previous source/line.
    ///
    /// May jump more than one constraint in case we have multiple constraints defined in a single
    /// source/file tuple.
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
            let different_line = source != current.name() || line != current.line();

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
            let different_line = source != current.name() || line != current.line();

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

    /// Attempt to jump to a given constraint
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
    /// May jump more than one constraint in case we have multiple constraints defined in a single
    /// source/file tuple.
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
            let different_line = source != current.name() || line != current.line();

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
            let different_line = source != current.name() || line != current.line();

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
