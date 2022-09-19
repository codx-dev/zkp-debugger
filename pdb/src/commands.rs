mod command;
mod instruction;

use std::io;

use rustyline::completion::FilenameCompleter;
use rustyline::hint::Hinter;
use rustyline::Context;
use rustyline_derive::{Completer, Helper, Highlighter, Validator};

pub use command::Command;
pub use instruction::Instruction;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParsedLine {
    Resolved {
        cmd: Command,
    },
    Completable {
        instruction: Instruction,
        completion: String,
    },
    Invalid,
    Empty,
}

/// Command parser for PDB
#[derive(Completer, Helper, Validator, Highlighter)]
pub struct CommandParser {
    instructions: Vec<Instruction>,
    filename_completer: FilenameCompleter,
}

impl CommandParser {
    /// Return all available instructions
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }
}

impl Clone for CommandParser {
    fn clone(&self) -> Self {
        Self {
            instructions: self.instructions.clone(),
            filename_completer: FilenameCompleter::new(),
        }
    }
}

impl Hinter for CommandParser {
    type Hint = String;

    fn hint(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &Context<'_>,
    ) -> Option<Self::Hint> {
        self.parse_completable(line).ok().and_then(|c| match c {
            ParsedLine::Completable { completion, .. } => Some(completion),
            _ => None,
        })
    }
}

impl Default for CommandParser {
    fn default() -> Self {
        Self {
            instructions: vec![
                Instruction::Afore,
                Instruction::Breakpoint,
                Instruction::Continue,
                Instruction::Delete,
                Instruction::Goto,
                Instruction::Help,
                Instruction::Next,
                Instruction::Open,
                Instruction::Print,
                Instruction::Restart,
                Instruction::Turn,
                Instruction::Quit,
                Instruction::Witness,
            ],
            filename_completer: FilenameCompleter::new(),
        }
    }
}

impl CommandParser {
    fn split(line: &str) -> io::Result<Vec<String>> {
        shellwords::split(line)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
    }

    /// Attempt to parse a command, providing completion information
    pub fn parse_completable(&self, line: &str) -> io::Result<ParsedLine> {
        let ends_with_space = line.ends_with(' ');
        let tokens = Self::split(line)?;

        if tokens.is_empty() {
            return Ok(ParsedLine::Empty);
        }

        if tokens.len() == 1 && !ends_with_space {
            match self.instructions.iter().enumerate().find_map(|(idx, ins)| {
                ins.complete_unary(&tokens[0]).map(|c| (idx, c))
            }) {
                Some((idx, completion)) if completion.is_empty() => {
                    match self.instructions[idx].resolve_unary() {
                        Some(cmd) => return Ok(ParsedLine::Resolved { cmd }),
                        None => {
                            return Ok(ParsedLine::Completable {
                                instruction: self.instructions[idx],
                                completion: " ".to_string(),
                            })
                        }
                    }
                }

                Some((idx, completion)) => {
                    return Ok(ParsedLine::Completable {
                        instruction: self.instructions[idx],
                        completion: completion.to_string(),
                    })
                }
                None => (),
            }
        }

        let instruction = match self
            .instructions
            .iter()
            .find(|i| i.complete_unary(&tokens[0]).is_some())
        {
            Some(i) => i,
            None => return Ok(ParsedLine::Invalid),
        };

        let arg = if tokens.len() == 1 { "" } else { &tokens[1] };
        if let Some(completion) = instruction.complete_binary(self, arg) {
            return Ok(ParsedLine::Completable {
                instruction: *instruction,
                completion,
            });
        }

        Ok(ParsedLine::Invalid)
    }

    /// Attempt to parse a command
    pub fn parse(&self, line: &str) -> io::Result<Command> {
        let tokens = Self::split(line)?;

        if tokens.is_empty() {
            return Ok(Command::Empty);
        }

        let instruction = match self
            .instructions
            .iter()
            .find(|i| i.complete_unary(&tokens[0]).is_some())
        {
            Some(i) => i,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "the provided instruction is invalid",
                ))
            }
        };

        if let Some(command) = instruction.resolve_unary() {
            return Ok(command);
        }

        if tokens.len() != 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "single argument expected. syntax: {}",
                    instruction.syntax()
                ),
            ));
        }

        Command::try_from_binary(instruction, &tokens[1])
    }
}

#[test]
fn validate_return_all_instructions() {
    let flag = 0b1111111111111;
    let result = CommandParser::default().instructions().iter().fold(
        0,
        |bit, instruction| match instruction {
            Instruction::Afore => bit | 0b1000000000000,
            Instruction::Breakpoint => bit | 0b0100000000000,
            Instruction::Continue => bit | 0b0010000000000,
            Instruction::Delete => bit | 0b0001000000000,
            Instruction::Goto => bit | 0b0000100000000,
            Instruction::Help => bit | 0b0000010000000,
            Instruction::Next => bit | 0b0000001000000,
            Instruction::Open => bit | 0b0000000100000,
            Instruction::Print => bit | 0b0000000010000,
            Instruction::Restart => bit | 0b0000000001000,
            Instruction::Turn => bit | 0b0000000000100,
            Instruction::Quit => bit | 0b0000000000010,
            Instruction::Witness => bit | 0b0000000000001,
        },
    );
    assert_eq!(flag, result);

    let a = CommandParser::default().instructions().to_vec();
    let mut b = CommandParser::default().instructions().to_vec();

    b.as_mut_slice().sort();
    b.dedup();

    assert_eq!(a.len(), b.len());
}
#[test]
fn validate_parse_completable() {
    let cases_instructions = vec![
        ("ope", "n", Instruction::Open),
        ("open", " ", Instruction::Open),
        ("open Carg", "o.toml", Instruction::Open),
    ];

    let cases_empty = vec![" "];

    let cases_invalid = vec!["Open "];

    let cases_instructions = cases_instructions.into_iter().map(
        |(input, completion, instruction)| {
            (
                input,
                ParsedLine::Completable {
                    instruction,
                    completion: completion.to_string(),
                },
            )
        },
    );

    let cases_empty = cases_empty
        .into_iter()
        .map(|input| (input, ParsedLine::Empty));

    let cases_invalid = cases_invalid
        .into_iter()
        .map(|input| (input, ParsedLine::Invalid));

    let parser = CommandParser::default();
    cases_instructions
        .chain(cases_empty)
        .chain(cases_invalid)
        .for_each(|(input, parsedline)| {
            let partial_parse = parser.parse_completable(input).unwrap();

            assert_eq!(partial_parse, parsedline);
        });
}

#[test]
fn validate_parse() {
    let parser = CommandParser::default();
    let cases_error = vec!["aaa", "open "];
    let cases_ok = vec!["", "quit", "open Cargo.toml"];

    for cases in cases_error.into_iter() {
        let result_err = parser.parse(cases);
        assert!(result_err.is_err());
    }

    for cases in cases_ok.into_iter() {
        let result_ok = parser.parse(cases);
        assert!(result_ok.is_ok());
    }
}
