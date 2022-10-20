use std::time;

use rustyline::Config as RustylineConfig;
use serde::{Deserialize, Serialize};
use toml_base_config::BaseConfig;

/// Readline configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Readline {
    max_history_size: usize,
    history_ignore_space: bool,
    completion_prompt_limit: usize,
    keyseq_timeout: i32,
    auto_add_history: bool,
    tab_stop: usize,
    indent_size: usize,
    check_cursor_position: bool,
}

impl From<Readline> for RustylineConfig {
    fn from(config: Readline) -> Self {
        let Readline {
            max_history_size,
            history_ignore_space,
            completion_prompt_limit,
            keyseq_timeout,
            auto_add_history,
            tab_stop,
            indent_size,
            check_cursor_position,
        } = config;

        RustylineConfig::builder()
            .max_history_size(max_history_size)
            .history_ignore_space(history_ignore_space)
            .completion_prompt_limit(completion_prompt_limit)
            .keyseq_timeout(keyseq_timeout)
            .auto_add_history(auto_add_history)
            .tab_stop(tab_stop)
            .indent_size(indent_size)
            .check_cursor_position(check_cursor_position)
            .build()
    }
}

impl Default for Readline {
    fn default() -> Self {
        let base = RustylineConfig::default();

        let max_history_size = 500;
        let auto_add_history = true;

        let history_ignore_space = base.history_ignore_space();
        let completion_prompt_limit = base.completion_prompt_limit();
        let keyseq_timeout = base.keyseq_timeout();
        let tab_stop = base.tab_stop();
        let indent_size = base.indent_size();
        let check_cursor_position = base.check_cursor_position();

        Self {
            max_history_size,
            history_ignore_space,
            completion_prompt_limit,
            keyseq_timeout,
            auto_add_history,
            tab_stop,
            indent_size,
            check_cursor_position,
        }
    }
}

/// Constraint renderization parameters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Render {
    pub margin: usize,
    pub header: bool,
    pub grid: bool,
    pub line_numbers: bool,
    pub theme: String,
}

impl Default for Render {
    fn default() -> Self {
        Self {
            margin: 10,
            header: true,
            grid: true,
            line_numbers: true,
            theme: match termbg::theme(time::Duration::from_millis(100)) {
                Ok(termbg::Theme::Light) => "gruvbox-light",
                _ => "gruvbox-dark",
            }
            .to_string(),
        }
    }
}

/// App configuration
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub readline: Readline,
    pub render: Render,
}

impl Config {
    /// Create an instance of the rustyline configuration
    pub fn rustyline(&self) -> RustylineConfig {
        self.readline.into()
    }
}

impl BaseConfig for Config {
    const PACKAGE: &'static str = env!("CARGO_PKG_NAME");
}
