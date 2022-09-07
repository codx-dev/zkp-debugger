use std::collections::HashMap;

use crate::Constraint;

/// Breakpoint definition
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Breakpoint {
    /// Source pattern that will trigger the breakpoint
    pub source: String,
    /// Line of the source that will trigger the breakpoin. If `None`, any incidence of `source`
    /// will trigger the breakpoint, regardless of the line.
    pub line: Option<u64>,
}

impl Breakpoint {
    /// Check if breakpoint matches the given arguments
    pub fn matches(&self, source: &str, line: u64) -> bool {
        source.contains(&self.source)
            && match self.line {
                Some(l) => l == line,
                None => true,
            }
    }
}

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

impl Breakpoints {
    pub fn add(&mut self, source: String, line: Option<u64>) -> usize {
        let breakpoint = Breakpoint { source, line };

        let id = *self.breakpoints.entry(breakpoint).or_insert(self.next_id);

        if id >= self.next_id {
            self.next_id += 1;
        }

        id
    }

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

    pub fn find_breakpoint<'a>(&self, constraint: &Constraint<'a>) -> Option<usize> {
        let source = constraint.name();
        let line = constraint.line();

        self.breakpoints
            .keys()
            .find(|b| b.matches(source, line))
            .and_then(|b| self.breakpoints.get(b).copied())
    }

    pub fn find_breakpoint_from_id(&self, id: usize) -> Option<&Breakpoint> {
        self.breakpoints
            .iter()
            .find_map(|(b, idx)| (id == *idx).then_some(b))
    }
}
