#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub name: String,
    pub contents: String,
    pub line: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Output {
    pub contents: Option<Source>,
    pub console: Vec<String>,
    pub error: Vec<String>,
}

impl Output {
    pub fn merge(&mut self, other: Self) {
        self.console.extend(other.console);
        self.error.extend(other.error);

        if let Some(c) = other.contents {
            self.contents.replace(c);
        }
    }
}
