use std::path::PathBuf;
use std::str::FromStr;
use std::{io, vec};

use dap_reactor::prelude::{
    Breakpoint, ContinueArguments, GotoArguments, InitializeArguments,
    ReverseContinueArguments, Source, StepBackArguments, VariablesArguments,
};
use dap_reactor::request::Request;
use dusk_cdf::ZkRequest;

use super::Instruction;

/// A PDB command
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Command {
    /// Execute the previous constraint
    Afore,
    /// Set a new breakpoint in the file that matches the given pattern
    Breakpoint {
        /// Source pattern
        source: String,
        /// Optional line. If empty, will stop whenever the source file is
        /// opened
        line: Option<u64>,
    },
    /// Continue the execution of the program
    Continue,
    /// Delete a breakpoint
    Delete {
        /// Id of the breakpoint
        id: usize,
    },
    /// Jump to a constraint
    Goto {
        /// Id of the constraint
        id: usize,
    },
    /// Print the help menu
    Help,
    /// Execute to next constraint
    Next,
    /// Open a CDF file
    Open {
        /// File path
        path: String,
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
        id: usize,
    },
}

impl Command {
    /// Attempt to parse a command from a binary tuple composed of an
    /// instruction and an argument
    pub fn try_from_binary(
        instruction: &Instruction,
        arg: &str,
    ) -> io::Result<Self> {
        match instruction {
            Instruction::Open => PathBuf::from(arg)
                .canonicalize()
                .map(|path| path.display().to_string())
                .map(|path| Self::Open { path }),

            Instruction::Breakpoint => {
                let mut args = arg.split(':');

                let source = args
                    .next()
                    .unwrap_or("split always generate a first element")
                    .into();

                let line = args.next().map(u64::from_str).transpose().map_err(
                    |e| io::Error::new(io::ErrorKind::InvalidInput, e),
                )?;

                Ok(Self::Breakpoint { source, line })
            }

            Instruction::Delete => usize::from_str(arg)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
                .map(|id| Self::Delete { id }),

            Instruction::Goto => usize::from_str(arg)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
                .map(|id| Self::Goto { id }),

            Instruction::Witness => usize::from_str(arg)
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

impl IntoIterator for Command {
    type Item = Request;
    type IntoIter = vec::IntoIter<Request>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Command::Afore => vec![Request::StepBack {
                arguments: StepBackArguments {
                    thread_id: 0,
                    single_thread: true,
                    granularity: None,
                },
            }]
            .into_iter(),

            Command::Breakpoint { source, line } => {
                vec![ZkRequest::AddBreakpoint {
                    breakpoint: Breakpoint {
                        id: None,
                        verified: true,
                        message: None,
                        source: Some(Source {
                            name: Some(source),
                            source_reference: None,
                            presentation_hint: None,
                            origin: None,
                            sources: vec![],
                            adapter_data: None,
                            checksums: vec![],
                        }),
                        line,
                        column: None,
                        end_line: line,
                        end_column: None,
                        instruction_reference: None,
                        offset: None,
                    },
                }
                .into()]
                .into_iter()
            }

            Command::Continue => vec![Request::Continue {
                arguments: ContinueArguments {
                    thread_id: 0,
                    single_thread: true,
                },
            }]
            .into_iter(),

            Command::Delete { id } => {
                vec![ZkRequest::RemoveBreakpoint { id: id as u64 }.into()]
                    .into_iter()
            }

            Command::Goto { id } => vec![Request::Goto {
                arguments: GotoArguments {
                    thread_id: 0,
                    target_id: id as u64,
                },
            }]
            .into_iter(),

            Command::Help => vec![].into_iter(),

            Command::Next => {
                vec![Request::Next { arguments: None }].into_iter()
            }

            Command::Open { .. } => vec![Request::Initialize {
                arguments: InitializeArguments {
                    client_id: None,
                    client_name: None,
                    adapter_id: "cdf".into(),
                    locale: None,
                    lines_start_at_1: true,
                    column_start_at_1: true,
                    path_format: None,
                    supports_variable_type: false,
                    supports_variable_paging: false,
                    supports_run_in_terminal_request: false,
                    supports_memory_references: false,
                    supports_progress_reporting: false,
                    supports_invalidated_event: false,
                    supports_memory_event: false,
                    supports_args_can_be_interpreted_by_shell: false,
                },
            }]
            .into_iter(),

            Command::Print => vec![Request::Variables {
                arguments: VariablesArguments {
                    variables_reference: 0,
                    filter: None,
                    start: None,
                    count: None,
                    format: None,
                },
            }]
            .into_iter(),

            Command::Restart => {
                vec![Request::Restart { arguments: None }].into_iter()
            }

            Command::Turn => vec![Request::ReverseContinue {
                arguments: ReverseContinueArguments {
                    thread_id: 0,
                    single_thread: true,
                },
            }]
            .into_iter(),

            Command::Quit => {
                vec![Request::Terminate { arguments: None }].into_iter()
            }

            Command::Witness { id } => {
                vec![ZkRequest::Witness { id }.into()].into_iter()
            }
        }
    }
}

#[test]
fn try_from_binary_wont_panic_with_unary() {
    Command::try_from_binary(&Instruction::Print, "xxx")
        .expect_err("Print shouldn't take arguments");
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

    let c = Command::Open {
        path: cargo.display().to_string(),
    };

    assert_eq!(c, command);
}

#[test]
fn try_from_binary_breakpoint_works() {
    let source = String::from("lib.rs");

    Command::try_from_binary(&Instruction::Breakpoint, "lib.rs:invalid_line")
        .expect_err("line must be numeric");

    let breakpoint =
        Command::try_from_binary(&Instruction::Breakpoint, &source)
            .expect("failed to create breakpoint command");

    let b = Command::Breakpoint {
        source: source.clone(),
        line: None,
    };

    assert_eq!(b, breakpoint);

    let line = 115;
    let breakpoint = Command::try_from_binary(
        &Instruction::Breakpoint,
        &format!("{}:{}", source, line),
    )
    .expect("failed to create breakpoint command");

    let b = Command::Breakpoint {
        source,
        line: Some(line),
    };

    assert_eq!(b, breakpoint);
}

#[test]
fn try_from_binary_delete_works() {
    Command::try_from_binary(&Instruction::Delete, "xx")
        .expect_err("delete should be numeric");

    let id = 2387;
    let delete =
        Command::try_from_binary(&Instruction::Delete, &format!("{}", id))
            .expect("failed to create delete command");
    let d = Command::Delete { id };

    assert_eq!(d, delete);
}

#[test]
fn try_from_binary_goto_works() {
    Command::try_from_binary(&Instruction::Goto, "xx")
        .expect_err("goto should be numeric");

    let id = 2387;
    let goto = Command::try_from_binary(&Instruction::Goto, &format!("{}", id))
        .expect("failed to create goto command");
    let g = Command::Goto { id };

    assert_eq!(g, goto);
}

#[test]
fn try_from_binary_witness_works() {
    Command::try_from_binary(&Instruction::Witness, "xx")
        .expect_err("witness should be numeric");

    let id = 2387;
    let witness =
        Command::try_from_binary(&Instruction::Witness, &format!("{}", id))
            .expect("failed to create witness command");
    let w = Command::Witness { id };

    assert_eq!(w, witness);
}

#[test]
fn command_generates_requests() {
    Command::Afore.into_iter().next().expect("req");
    Command::Breakpoint {
        source: "foo".into(),
        line: None,
    }
    .into_iter()
    .next()
    .expect("req");
    Command::Continue.into_iter().next().expect("req");
    Command::Delete { id: 83 }.into_iter().next().expect("req");
    Command::Goto { id: 83 }.into_iter().next().expect("req");
    Command::Next.into_iter().next().expect("req");
    Command::Open { path: "foo".into() }
        .into_iter()
        .next()
        .expect("req");
    Command::Print.into_iter().next().expect("req");
    Command::Restart.into_iter().next().expect("req");
    Command::Quit.into_iter().next().expect("req");
    Command::Witness { id: 83 }.into_iter().next().expect("req");

    assert!(Command::Help.into_iter().next().is_none());
}
