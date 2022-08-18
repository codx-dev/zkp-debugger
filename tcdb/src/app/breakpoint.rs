use std::collections::HashMap;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Breakpoint {
    source: String,
    line: Option<u64>,
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

    pub fn is_breakpoint(&self, source: &str, line: u64) -> bool {
        self.breakpoints.keys().any(|b| {
            source.contains(&b.source)
                && match b.line {
                    Some(l) => l == line,
                    None => true,
                }
        })
    }
}
