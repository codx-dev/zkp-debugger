use std::io;

use dap_reactor::prelude::Breakpoint;
use dap_reactor::response::Response;
use dap_reactor::{reactor::ClientRequest, request::Request};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Witness;

use super::utils;

fn err(e: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, e)
}

/// A request customized for the ZK backend
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZkRequest {
    /// Add a new breakpoint
    AddBreakpoint {
        /// Breakpoint to be added
        breakpoint: Breakpoint,
    },
    /// Remove a previously added breakpoint
    RemoveBreakpoint {
        /// Id of the breakpoint
        id: u64,
    },
    /// Load a CDF file
    LoadCdf {
        /// Path of the CDF file to be loaded
        path: String,
    },
    /// Request the source contents of the CDF file
    SourceContents,
    /// Return the internal data of a witness
    Witness {
        /// Id of the witness
        id: usize,
    },
}

impl From<ZkRequest> for Request {
    fn from(request: ZkRequest) -> Self {
        match request {
            ZkRequest::AddBreakpoint { breakpoint } => Request::Custom {
                arguments: Some(serde_json::json!({
                    "command": "addBreakpoint",
                    "breakpoint": Value::from(breakpoint),
                })),
            },

            ZkRequest::RemoveBreakpoint { id } => Request::Custom {
                arguments: Some(serde_json::json!({
                    "command": "removeBreakpoint",
                    "id": id,
                })),
            },

            ZkRequest::LoadCdf { path } => Request::Custom {
                arguments: Some(serde_json::json!({
                    "command": "loadCdf",
                    "path": path,
                })),
            },

            ZkRequest::SourceContents => Request::Custom {
                arguments: Some(serde_json::json!({
                    "command": "sourceContents",
                })),
            },

            ZkRequest::Witness { id } => Request::Custom {
                arguments: Some(serde_json::json!({
                    "command": "witness",
                    "id": id,
                })),
            },
        }
    }
}

impl From<ZkRequest> for ClientRequest {
    fn from(value: ZkRequest) -> Self {
        Request::from(value).into()
    }
}

impl TryFrom<Option<&Value>> for ZkRequest {
    type Error = io::Error;

    fn try_from(arguments: Option<&Value>) -> io::Result<Self> {
        let args = arguments
            .and_then(Value::as_object)
            .ok_or_else(|| err("arguments should be an object"))?;

        let command = args
            .get("command")
            .and_then(Value::as_str)
            .ok_or_else(|| err("arguments should contain a command"))?;

        match command {
            "addBreakpoint" => args
                .get("breakpoint")
                .and_then(Value::as_object)
                .map(Breakpoint::try_from)
                .transpose()?
                .map(|breakpoint| ZkRequest::AddBreakpoint { breakpoint })
                .ok_or_else(|| err("invalid breakpoint attribute")),

            "removeBreakpoint" => args
                .get("id")
                .and_then(Value::as_u64)
                .map(|id| ZkRequest::RemoveBreakpoint { id })
                .ok_or_else(|| err("invalid id attribute")),

            "loadCdf" => args
                .get("path")
                .and_then(Value::as_str)
                .map(|path| ZkRequest::LoadCdf { path: path.into() })
                .ok_or_else(|| err("invalid path attribute")),

            "sourceContents" => Ok(ZkRequest::SourceContents),

            "witness" => args
                .get("id")
                .and_then(Value::as_u64)
                .map(|id| ZkRequest::Witness { id: id as usize })
                .ok_or_else(|| err("invalid id attribute")),

            _ => Err(io::Error::new(io::ErrorKind::Other, "unknown command")),
        }
    }
}

/// Source representation in the ZK DAP backend
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZkSource {
    /// Path identifier.
    ///
    /// Won't necessarily reflect a real path in the disk.
    pub path: String,
    /// Source contents
    pub contents: String,
}

/// Witness representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkWitness {
    /// Id of the witness
    pub id: usize,
    /// Associated constraint with the creation of the witness
    pub constraint: Option<usize>,
    /// Allocated value
    pub value: String,
    /// Source name associated with the witness declaration
    pub source: String,
    /// Source line associated with the witness declaration
    pub line: u64,
}

impl From<ZkWitness> for Value {
    fn from(value: ZkWitness) -> Self {
        let ZkWitness {
            id,
            constraint,
            value,
            source,
            line,
        } = value;

        serde_json::json!({
            "id": id,
            "constraint": constraint,
            "value": value,
            "source": source,
            "line": line,
        })
    }
}

impl From<Witness<'_>> for ZkWitness {
    fn from(w: Witness) -> Self {
        Self {
            id: w.id(),
            constraint: w.constraint(),
            value: utils::scalar_to_string(w.value()),
            source: w.name().to_string(),
            line: w.line(),
        }
    }
}

impl TryFrom<&Value> for ZkWitness {
    type Error = io::Error;

    fn try_from(v: &Value) -> Result<Self, Self::Error> {
        let id = v
            .get("id")
            .and_then(Value::as_u64)
            .map(|id| id as usize)
            .ok_or_else(|| err("id expected as number"))?;

        let constraint = v
            .get("constraint")
            .and_then(Value::as_u64)
            .map(|c| c as usize);

        let value = v
            .get("value")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| err("value expected as string"))?;

        let source = v
            .get("source")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| err("source expected as string"))?;

        let line = v
            .get("line")
            .and_then(Value::as_u64)
            .ok_or_else(|| err("line expected as number"))?;

        Ok(Self {
            id,
            constraint,
            value,
            source,
            line,
        })
    }
}

/// A response produced by the ZK DAP backend
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZkResponse {
    /// A breakpoint was added
    AddBreakpoint {
        /// Id of the added breakpoint
        id: u64,
    },
    /// Remove a previously added breakpoint
    RemoveBreakpoint {
        /// Id of the removed breakpoint
        id: u64,
        /// Flag on whether or not the breakpoint was existent and removed
        removed: bool,
    },
    /// The CDF file was loaded
    LoadCdf,
    /// List of sources contained in the loaded CDF file
    SourceContents {
        /// Sources list
        sources: Vec<ZkSource>,
    },
    /// Internal data of a witness evaluated
    Witness {
        /// Evaluated data
        witness: ZkWitness,
    },
}

impl From<ZkResponse> for Response {
    fn from(response: ZkResponse) -> Self {
        match response {
            ZkResponse::AddBreakpoint { id } => Response::Custom {
                body: Some(serde_json::json!({
                    "command": "addBreakpoint",
                    "id": id,
                })),
            },

            ZkResponse::RemoveBreakpoint { id, removed } => Response::Custom {
                body: Some(serde_json::json!({
                    "command": "removeBreakpoint",
                    "id": id,
                    "removed": removed,
                })),
            },

            ZkResponse::LoadCdf => Response::Custom {
                body: Some(serde_json::json!({
                    "command": "loadCdf",
                })),
            },

            ZkResponse::SourceContents { sources } => Response::Custom {
                body: Some(serde_json::json!({
                    "command": "sourceContents",
                    "sources": sources,
                })),
            },

            ZkResponse::Witness { witness } => Response::Custom {
                body: Some(serde_json::json!({
                    "command": "witness",
                    "witness": Value::from(witness),
                })),
            },
        }
    }
}

impl TryFrom<Option<&Value>> for ZkResponse {
    type Error = io::Error;

    fn try_from(body: Option<&Value>) -> io::Result<Self> {
        let body = body
            .and_then(Value::as_object)
            .ok_or_else(|| err("body should be an object"))?;

        let command = body
            .get("command")
            .and_then(Value::as_str)
            .ok_or_else(|| err("body should contain a command"))?;

        match command {
            "addBreakpoint" => body
                .get("id")
                .and_then(Value::as_u64)
                .map(|id| ZkResponse::AddBreakpoint { id })
                .ok_or_else(|| err("invalid id attribute")),

            "removeBreakpoint" => {
                let id = body
                    .get("id")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| err("invalid id attribute"))?;

                let removed = body
                    .get("removed")
                    .and_then(Value::as_bool)
                    .ok_or_else(|| err("invalid removed attribute"))?;

                Ok(Self::RemoveBreakpoint { id, removed })
            }

            "loadCdf" => Ok(Self::LoadCdf),

            "sourceContents" => body
                .get("sources")
                .and_then(Value::as_array)
                .ok_or_else(|| err("invalid sources attribute"))?
                .iter()
                .map(|s| {
                    ZkSource::deserialize(s)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                })
                .collect::<io::Result<_>>()
                .map(|sources| Self::SourceContents { sources }),

            "witness" => body
                .get("witness")
                .ok_or_else(|| err("witness is mandatory"))
                .and_then(ZkWitness::try_from)
                .map(|witness| Self::Witness { witness }),

            _ => Err(io::Error::new(io::ErrorKind::Other, "unknown command")),
        }
    }
}
