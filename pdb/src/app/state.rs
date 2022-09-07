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
