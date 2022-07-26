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

    fn hint(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
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
        shellwords::split(line).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
    }

    /// Attempt to parse a command, providing completion information
    pub fn parse_completable(&self, line: &str) -> io::Result<ParsedLine> {
        let ends_with_space = line.ends_with(' ');
        let tokens = Self::split(line)?;

        if tokens.is_empty() {
            return Ok(ParsedLine::Empty);
        }

        if tokens.len() == 1 && !ends_with_space {
            match self
                .instructions
                .iter()
                .enumerate()
                .find_map(|(idx, ins)| ins.complete_unary(&tokens[0]).map(|c| (idx, c)))
            {
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
                format!("single argument expected. syntax: {}", instruction.syntax()),
            ));
        }

        Command::try_from_binary(instruction, &tokens[1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validate_return_all_instructions() {
        let default = &CommandParser::default();
        let default_val = CommandParser::instructions(default);
        let vec_actions = vec![
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
        ];
        assert_eq!(default_val, vec_actions);
    }

    #[test]
    fn validade_clone_intructinos() {
        let default = &CommandParser::default();
        let cloned = &CommandParser::clone(&CommandParser::default());
        assert_eq!(default.instructions, cloned.instructions);
    }

    #[test]
    fn validade_parse_completable() {
        //Complete comand
        let partial_parse =
            CommandParser::parse_completable(&CommandParser::default(), "ope").unwrap();
        let want = ParsedLine::Completable {
            instruction: (Instruction::Open),
            completion: ("n".to_string()),
        };
        assert_eq!(partial_parse, want);

        //Comand completed, return space
        let partial_parse =
            CommandParser::parse_completable(&CommandParser::default(), "open").unwrap();
        let want = ParsedLine::Completable {
            instruction: (Instruction::Open),
            completion: (" ".to_string()),
        };
        assert_eq!(partial_parse, want);

        //Token epmty
        let partial_parse =
            CommandParser::parse_completable(&CommandParser::default(), "").unwrap();
        let want = ParsedLine::Empty;
        assert_eq!(partial_parse, want);

        //Token space
        let partial_parse =
            CommandParser::parse_completable(&CommandParser::default(), " ").unwrap();
        let want = ParsedLine::Empty;
        assert_eq!(partial_parse, want);

        //hint cargo.toml
        let partial_parse =
            CommandParser::parse_completable(&CommandParser::default(), "open Carg").unwrap();
        let want = ParsedLine::Completable {
            instruction: (Instruction::Open),
            completion: ("o.toml".to_string()),
        };
        assert_eq!(partial_parse, want);

        //Invalid
        let partial_parse =
            CommandParser::parse_completable(&CommandParser::default(), "Open ").unwrap();
        let want = ParsedLine::Invalid;
        assert_eq!(partial_parse, want);
    }
}
