use std::collections::HashMap;
use std::ops::Deref;

use crate::Constraint;

/// A single breakpoint in code. A `Breakpoint` has a source pattern which
/// triggers the breakpoint and the line number.
///
/// The [`ZkDebugger`](struct.ZkDebugger.html) struct stores the breakpoints for
/// debugging.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Breakpoint {
    /// Source pattern that will trigger the breakpoint.
    pub source: String,
    /// Line of the source that will trigger the breakpoint. If `None`, any
    /// incidence of `source` will trigger the breakpoint, regardless of
    /// the line.
    pub line: Option<u64>,
}

impl Breakpoint {
    /// Check if the source and line number matches with the breakpoint.
    ///
    /// # Example
    ///
    /// ```
    /// # use dusk_cdf::Breakpoint;
    /// let breakpoint = Breakpoint {
    ///     source: String::from("xyz"),
    ///     line: Some(40),
    /// };
    ///
    /// assert!(breakpoint.matches("xyz", 40));
    /// ```
    pub fn matches(&self, source: &str, line: u64) -> bool {
        source.contains(&self.source)
            && match self.line {
                Some(l) => l == line,
                None => true,
            }
    }
}

/// A collection of breakpoints, the debugger keeps track of the breakpoints
/// using this struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Breakpoints {
    next_id: usize,
    breakpoints: HashMap<Breakpoint, usize>,
}

impl Default for Breakpoints {
    fn default() -> Self {
        Self {
            next_id: 1,
            breakpoints: HashMap::default(),
        }
    }
}

impl Deref for Breakpoints {
    type Target = HashMap<Breakpoint, usize>;

    fn deref(&self) -> &Self::Target {
        &self.breakpoints
    }
}

impl Breakpoints {
    /// Add a breakpoint to the collection of breakpoints.
    pub fn add(&mut self, source: String, line: Option<u64>) -> usize {
        let breakpoint = Breakpoint { source, line };

        let id = *self.breakpoints.entry(breakpoint).or_insert(self.next_id);

        if id >= self.next_id {
            self.next_id += 1;
        }

        id
    }

    /// Remove a breakpoint from the collection of breakpoints.
    pub fn remove(&mut self, id: usize) -> Option<Breakpoint> {
        let removed = self
            .breakpoints
            .iter()
            .find_map(|(breakpoint, idx)| (idx == &id).then_some(breakpoint))
            .cloned();

        if let Some(b) = &removed {
            self.breakpoints.remove(b);
        }

        removed
    }

    /// Find a breakpoint from the collection of breakpoints given constraint.
    /// The name of the constraint is used as the source pattern
    pub fn find_breakpoint<'a>(
        &self,
        constraint: &Constraint<'a>,
    ) -> Option<usize> {
        let source = constraint.name();
        let line = constraint.line();

        self.breakpoints
            .keys()
            .find(|b| b.matches(source, line))
            .and_then(|b| self.breakpoints.get(b).copied())
    }

    /// Find a breakpoint by its id.
    pub fn find_breakpoint_from_id(&self, id: usize) -> Option<&Breakpoint> {
        self.breakpoints
            .iter()
            .find_map(|(b, idx)| (id == *idx).then_some(b))
    }

    /// Clear all breakpoints that matches the given source
    pub fn clear(&mut self, source: &str) {
        self.breakpoints
            .retain(|b, _| !source.contains(b.source.as_str()));
    }
}
