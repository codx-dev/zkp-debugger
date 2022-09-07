/// State describind a mutation of the zk debugger
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum State {
    /// BOF of the CDF backend
    ///
    /// Id is constant `0`
    Beginning,
    /// Mutated the position to a different constraint
    Constraint {
        /// Id of the constraint
        id: usize,
    },
    /// Hit a constraint that evaluated to false
    InvalidConstraint {
        /// Id of the constraint
        id: usize,
    },
    /// Hit a breakpoint
    Breakpoint {
        /// Id of the breakpoint
        id: usize,
    },
    /// EOF of the CDF backend
    End {
        /// Id of the breakpoint
        id: usize,
    },
}
