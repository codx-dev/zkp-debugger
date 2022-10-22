use super::{Command, CommandParser};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Instruction {
    Afore = 0x00,
    Breakpoint = 0x01,
    Continue = 0x02,
    Delete = 0x03,
    Goto = 0x04,
    Help = 0x05,
    Next = 0x06,
    Open = 0x07,
    Print = 0x08,
    Restart = 0x09,
    Turn = 0x0a,
    Quit = 0x0b,
    Witness = 0x0c,
}

impl Instruction {
    pub fn help(&self) -> &'static str {
        match self {
            Instruction::Afore => "go to the previous constraint",
            Instruction::Breakpoint => "set a new breakpoint. the name pattern doesn't have to be an exact match to the source name.",
            Instruction::Continue => "continue normal execution until next error",
            Instruction::Delete => "remove a breakpoint.",
            Instruction::Goto => "jump to a constraint",
            Instruction::Help => "print the help menu",
            Instruction::Next => "go to the next constraint",
            Instruction::Open => "open a file",
            Instruction::Print => "print constraint data",
            Instruction::Restart => "restart the execution of a circuit",
            Instruction::Turn => "reverse the execution of the circuit",
            Instruction::Quit => "terminate the session",
            Instruction::Witness => "print information about a witness",
        }
    }

    pub fn syntax(&self) -> &'static str {
        match self {
            Instruction::Afore => "afore",
            Instruction::Breakpoint => "breakpoint <NAME>[:LINE]",
            Instruction::Continue => "continue",
            Instruction::Delete => "delete <NUMBER>",
            Instruction::Goto => "goto <NUMBER>",
            Instruction::Help => "help",
            Instruction::Next => "next",
            Instruction::Open => "open <FILE>",
            Instruction::Print => "print",
            Instruction::Restart => "restart",
            Instruction::Turn => "turn",
            Instruction::Quit => "quit",
            Instruction::Witness => "witness <NUMBER>",
        }
    }

    pub fn token(&self) -> &'static str {
        match self {
            Instruction::Afore => "afore",
            Instruction::Breakpoint => "breakpoint",
            Instruction::Continue => "continue",
            Instruction::Delete => "delete",
            Instruction::Goto => "goto",
            Instruction::Help => "help",
            Instruction::Next => "next",
            Instruction::Open => "open",
            Instruction::Print => "print",
            Instruction::Restart => "restart",
            Instruction::Turn => "turn",
            Instruction::Quit => "quit",
            Instruction::Witness => "witness",
        }
    }

    pub fn complete_unary(&self, token: &str) -> Option<&str> {
        self.token()
            .starts_with(token)
            .then(|| self.token())
            .map(|t| {
                if t.len() == token.len() {
                    ""
                } else {
                    &t[token.len()..]
                }
            })
    }

    pub fn complete_binary(
        &self,
        parser: &CommandParser,
        token: &str,
    ) -> Option<String> {
        match self {
            Instruction::Open => parser
                .filename_completer
                .complete_path(token, token.len())
                .ok()
                .and_then(|(_, pairs)| {
                    pairs
                        .first()
                        .map(|pair| pair.replacement[token.len()..].to_string())
                }),

            _ => None,
        }
    }

    /// Resolve an unary token
    pub fn resolve_unary(&self) -> Option<Command> {
        match self {
            Instruction::Afore => Some(Command::Afore),
            Instruction::Continue => Some(Command::Continue),
            Instruction::Help => Some(Command::Help),
            Instruction::Next => Some(Command::Next),
            Instruction::Print => Some(Command::Print),
            Instruction::Restart => Some(Command::Restart),
            Instruction::Turn => Some(Command::Turn),
            Instruction::Quit => Some(Command::Quit),
            _ => None,
        }
    }
}

#[test]
fn complete_unary_works() {
    vec![
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
    ]
    .into_iter()
    .for_each(|t| {
        let token = t.token();

        (1..token.len()).for_each(|i| {
            let c = t
                .complete_unary(&token[0..i])
                .expect("failed to fetch token completion");

            assert_eq!(&token[i..], c);
        });
    });
}

#[test]
fn complete_binary_works() {
    use std::path::PathBuf;

    let manifest = env!("CARGO_MANIFEST_DIR");
    let cargo = PathBuf::from(manifest).join("Cargo.t");
    let cargo = cargo.to_str().expect("failed to fetch str from path");

    let parser = CommandParser::default();

    let completion = Instruction::Open
        .complete_binary(&parser, cargo)
        .expect("failed to fetch file completer");

    assert_eq!("oml", completion);
}

#[test]
fn help_generates_output() {
    Instruction::Afore.help();
    Instruction::Breakpoint.help();
    Instruction::Continue.help();
    Instruction::Delete.help();
    Instruction::Goto.help();
    Instruction::Help.help();
    Instruction::Next.help();
    Instruction::Open.help();
    Instruction::Print.help();
    Instruction::Restart.help();
    Instruction::Turn.help();
    Instruction::Quit.help();
    Instruction::Witness.help();

    Instruction::Afore.syntax();
    Instruction::Breakpoint.syntax();
    Instruction::Continue.syntax();
    Instruction::Delete.syntax();
    Instruction::Goto.syntax();
    Instruction::Help.syntax();
    Instruction::Next.syntax();
    Instruction::Open.syntax();
    Instruction::Print.syntax();
    Instruction::Restart.syntax();
    Instruction::Turn.syntax();
    Instruction::Quit.syntax();
    Instruction::Witness.syntax();
}

#[test]
fn unary_resolves() {
    Instruction::Quit
        .resolve_unary()
        .expect("failed to resolve unary");
}
