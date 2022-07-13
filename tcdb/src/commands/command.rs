use std::io;
use std::path::PathBuf;
use std::str::FromStr;

use super::Instruction;

/// A PDF command
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Command {
    /// Execute the previous constraint
    Afore,
    /// Set a new breakpoint in the file that matches the given pattern
    Breakpoint {
        /// Source pattern
        source: String,
        /// Optional line. If empty, will stop whenever the source file is opened
        line: Option<u64>,
    },
    /// Continue the execution of the program
    Continue,
    /// Delete a breakpoint
    Delete {
        /// Id of the breakpoint
        id: u64,
    },
    /// Empty command
    Empty,
    /// Jump to a constraint
    Goto {
        /// Id of the constraint
        id: u64,
    },
    /// Print the help menu
    Help,
    /// Execute to next constraint
    Next,
    /// Open a CDF file
    Open {
        /// File path
        path: PathBuf,
    },
    /// Print constraint data
    Print,
    /// Restart the execution of a circuit
    Restart,
    /// Reverse the execution of a circuit
    Turn,
    /// Quit the debugger
    Quit,
    /// Print information about a witness
    Witness {
        /// Id of the witness
        id: u64,
    },
}

impl Command {
    /// Attempt to parse a command from a binary tuple composed of an instruction and an argument
    pub fn try_from_binary(instruction: &Instruction, arg: &str) -> io::Result<Self> {
        match instruction {
            Instruction::Open => PathBuf::from(arg)
                .canonicalize()
                .map(|path| Self::Open { path }),

            Instruction::Breakpoint => {
                let mut args = arg.split(':');

                let source = args
                    .next()
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!(
                                "the file argument is mandatory for breakpoints. syntax: {}",
                                instruction.syntax()
                            ),
                        )
                    })
                    .map(String::from)?;

                let line = args
                    .next()
                    .map(u64::from_str)
                    .transpose()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

                Ok(Self::Breakpoint { source, line })
            }

            Instruction::Delete => u64::from_str(arg)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
                .map(|id| Self::Delete { id }),

            Instruction::Goto => u64::from_str(arg)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
                .map(|id| Self::Goto { id }),

            Instruction::Witness => u64::from_str(arg)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
                .map(|id| Self::Witness { id }),

            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "the provided instruction is unary. syntax: {}",
                    instruction.syntax()
                ),
            )),
        }
    }
}

#[test]
fn try_from_binary_open_works() {
    use std::path::PathBuf;

    let manifest = env!("CARGO_MANIFEST_DIR");
    let cargo = PathBuf::from(manifest)
        .join("Cargo.toml")
        .canonicalize()
        .expect("failed to canonicalize cargo path");

    let cargo_str = cargo.to_str().expect("failed to fetch str from path");
    let command = Command::try_from_binary(&Instruction::Open, cargo_str)
        .expect("failed to create open command");

    let c = Command::Open { path: cargo };

    assert_eq!(c, command);
}

#[test]
fn try_from_binary_breakpoint_works() {
    let source = String::from("lib.rs");

    let breakpoint = Command::try_from_binary(&Instruction::Breakpoint, &source)
        .expect("failed to create breakpoint command");

    let b = Command::Breakpoint {
        source: source.clone(),
        line: None,
    };

    assert_eq!(b, breakpoint);

    let line = 115;
    let breakpoint =
        Command::try_from_binary(&Instruction::Breakpoint, &format!("{}:{}", source, line))
            .expect("failed to create breakpoint command");

    let b = Command::Breakpoint {
        source,
        line: Some(line),
    };

    assert_eq!(b, breakpoint);
}

#[test]
fn try_from_binary_delete_works() {
    let id = 2387;
    let delete = Command::try_from_binary(&Instruction::Delete, &format!("{}", id))
        .expect("failed to create delete command");
    let d = Command::Delete { id };

    assert_eq!(d, delete);
}

#[test]
fn try_from_binary_goto_works() {
    let id = 2387;
    let goto = Command::try_from_binary(&Instruction::Goto, &format!("{}", id))
        .expect("failed to create goto command");
    let g = Command::Goto { id };

    assert_eq!(g, goto);
}

#[test]
fn try_from_binary_witness_works() {
    let id = 2387;
    let witness = Command::try_from_binary(&Instruction::Witness, &format!("{}", id))
        .expect("failed to create witness command");
    let w = Command::Witness { id };

    assert_eq!(w, witness);
}
