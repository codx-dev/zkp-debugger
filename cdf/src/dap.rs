//! Debug Adapter Protocol provider

use std::fs::File;
use std::io;
use std::net::SocketAddr;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

use dap_reactor::models::{ContinueResponse, CustomAddBreakpointArguments};
use dap_reactor::prelude::*;
use dap_reactor::types::Breakpoint;
use tokio::net;
use tokio::sync::Mutex;

use crate::{EncodableConstraint, EncodableWitness, State, ZkDebugger};

/// Evaluate expression to be consumed when creating a [`Request::Evaluate`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZkEvaluate {
    /// Evaluate a constraint with the provided index
    Constraint {
        /// Id of the constraint
        id: usize,
    },
    /// Evaluate the current constraint
    CurrentConstraint,
    /// Evaluate a witness with the provided index
    Witness {
        /// Id of the witness
        id: usize,
    },
}

impl From<ZkEvaluate> for String {
    fn from(ev: ZkEvaluate) -> Self {
        match ev {
            ZkEvaluate::Constraint { id } => format!("c{}", id),
            ZkEvaluate::CurrentConstraint => "x".into(),
            ZkEvaluate::Witness { id } => format!("w{}", id),
        }
    }
}

impl TryFrom<&str> for ZkEvaluate {
    type Error = io::Error;

    fn try_from(s: &str) -> io::Result<Self> {
        match s.split_at(0) {
            ("c", n) => n
                .parse::<usize>()
                .map(|id| Self::Constraint { id })
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e)),
            ("x", "") => Ok(Self::CurrentConstraint),
            ("w", n) => n
                .parse::<usize>()
                .map(|id| Self::Witness { id })
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e)),

            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "the provided evaluate request isn't valid",
            )),
        }
    }
}

/// Path & line representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkSourceDescription {
    /// Path of the source, as string
    pub name: String,
    /// Line number
    pub line: u64,
}

impl From<ZkSourceDescription> for String {
    fn from(source: ZkSourceDescription) -> Self {
        let ZkSourceDescription { name, line } = source;

        format!("{}:{}", name, line)
    }
}

impl TryFrom<&str> for ZkSourceDescription {
    type Error = io::Error;

    fn try_from(s: &str) -> io::Result<Self> {
        let (name, line) = s.split_once(':').ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid format for source description",
            )
        })?;

        let name = String::from(name);
        let line = line
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        Ok(Self { name, line })
    }
}

/// Builder for the [`ZkDap`] service
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZkDapBuilder<S> {
    /// Capacity of the internal channels of the service
    pub capacity: usize,
    /// Sockets to bind the service
    pub socket: S,
}

impl<S> ZkDapBuilder<S> {
    /// Initiate a default builder with the provided socket
    pub fn new(socket: S) -> Self {
        Self {
            capacity: 50,
            socket,
        }
    }

    /// Override the default channels capacity
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }
}

impl<S> ZkDapBuilder<S>
where
    S: net::ToSocketAddrs,
{
    /// Bind the [`ZkDebugger`] via DAP to a given socket
    pub async fn build(self) -> io::Result<ZkDapService> {
        let Self { capacity, socket } = self;

        let reactor =
            Reactor::new().with_capacity(capacity).bind(socket).await?;

        Ok(ZkDapService { reactor })
    }
}

/// Zk DAP reactor listener
pub struct ZkDapService {
    reactor: ReactorListener<ZkDap>,
}

impl Deref for ZkDapService {
    type Target = ReactorListener<ZkDap>;

    fn deref(&self) -> &Self::Target {
        &self.reactor
    }
}

impl ZkDapService {
    /// Listen for incoming connections to provide the DAP service
    pub async fn listen(self) -> io::Result<()> {
        self.reactor.listen().await
    }
}

/// Debug adapter protocol provider for the [`ZkDebugger`]
pub struct ZkDap {
    events: Sender<Event>,
    backend: Arc<Mutex<Option<ZkDebugger<File>>>>,
}

impl ZkDap {
    /// Define the implementation capabilities
    pub fn capabilities() -> Capabilities {
        Capabilities {
            supports_configuration_done_request: true,
            supports_function_breakpoints: true,
            supports_conditional_breakpoints: true,
            supports_hit_conditional_breakpoints: true,
            supports_evaluate_for_hovers: true,
            exception_breakpoint_filters: vec![],
            supports_step_back: true,
            supports_set_variable: false,
            supports_restart_frame: false,
            supports_goto_targets_request: false,
            supports_step_in_targets_request: false,
            supports_completions_request: false,
            completion_trigger_characters: vec![],
            supports_modules_request: false,
            additional_module_columns: vec![],
            supported_checksum_algorithms: vec![
                ChecksumAlgorithm::Md5,
                ChecksumAlgorithm::Sha1,
                ChecksumAlgorithm::Sha256,
                ChecksumAlgorithm::Timestamp,
            ],
            supports_restart_request: true,
            supports_exception_options: false,
            supports_value_formatting_options: false,
            supports_exception_info_request: false,
            support_terminate_debuggee: true,
            support_suspend_debuggee: true,
            supports_delayed_stack_trace_loading: false,
            supports_loaded_sources_request: true,
            supports_log_points: true,
            supports_terminate_threads_request: true,
            supports_set_expression: false,
            supports_terminate_request: true,
            supports_data_breakpoints: true,
            supports_read_memory_request: false,
            supports_write_memory_request: false,
            supports_disassemble_request: false,
            supports_cancel_request: false,
            supports_breakpoint_locations_request: true,
            supports_clipboard_context: false,
            supports_stepping_granularity: false,
            supports_instruction_breakpoints: false,
            supports_exception_filter_options: false,
            supports_single_thread_execution_requests: true,
        }
    }

    /// Bind the [`ZkDebugger`] via DAP to a given socket
    pub async fn bind<S>(capacity: usize, socket: S) -> io::Result<SocketAddr>
    where
        S: net::ToSocketAddrs,
    {
        let reactor = Reactor::<Self>::new()
            .with_capacity(capacity)
            .bind(socket)
            .await?;

        let socket = reactor.local_addr()?;

        tokio::spawn(async move {
            if let Err(e) = reactor.listen().await {
                tracing::error!("error listening to dap: {}", e);
            }
        });

        Ok(socket)
    }

    fn not_initialized() -> io::Error {
        io::Error::new(
            io::ErrorKind::Other,
            "the debugger is not initialized with a CDF file",
        )
    }

    async fn update_constraint(
        &self,
        debugger: &mut ZkDebugger<File>,
        reason: StoppedReason,
        breakpoints: Vec<usize>,
    ) -> io::Result<()> {
        let constraint = debugger.fetch_current_constraint()?;
        let description = ZkSourceDescription {
            name: constraint.name().into(),
            line: constraint.line(),
        };

        let description = Some(description.into());

        self.events
            .send(Event::Stopped {
                reason,
                description,
                thread_id: None,
                preserve_focus_hint: false,
                text: None,
                all_threads_stopped: true,
                hit_breakpoint_ids: breakpoints,
            })
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }

    async fn consume_state(
        &self,
        debugger: &mut ZkDebugger<File>,
        state: State,
    ) -> io::Result<()> {
        let (reason, breakpoints) = match state {
            State::Beginning => (StoppedReason::Custom("bof".into()), vec![]),
            State::Constraint { id: _ }
            | State::InvalidConstraint { id: _ } => {
                (StoppedReason::Exception, vec![])
            }
            State::Breakpoint { id } => (StoppedReason::Breakpoint, vec![id]),
            State::End { id: _ } => {
                (StoppedReason::Custom("eof".into()), vec![])
            }
        };

        self.update_constraint(debugger, reason, breakpoints)
            .await?;

        Ok(())
    }

    async fn breakpoint_locations(
        &self,
        arguments: Option<BreakpointLocationsArguments>,
    ) -> io::Result<Response> {
        let debugger = self.backend.lock().await;
        let debugger = debugger.as_ref().ok_or_else(Self::not_initialized)?;

        let breakpoints = debugger
            .breakpoints()
            .iter()
            .filter_map(|(b, _)| {
                arguments
                    .as_ref()
                    .map(
                        |BreakpointLocationsArguments {
                             source, line, ..
                         }| {
                            b.matches(
                                source.name.as_deref().unwrap_or(""),
                                *line,
                            )
                            .then_some(
                                BreakpointLocation {
                                    line: *line,
                                    column: None,
                                    end_line: None,
                                    end_column: None,
                                },
                            )
                        },
                    )
                    .unwrap_or_else(|| {
                        Some(BreakpointLocation {
                            line: b.line.unwrap_or_default(),
                            column: None,
                            end_line: None,
                            end_column: None,
                        })
                    })
            })
            .collect();

        // TODO https://github.com/codx-dev/dap-reactor/issues/12
        Ok(Response::BreakpointLocations {
            body: Some(BreakpointLocationsResponse { breakpoints }),
        })
    }

    async fn evaluate(
        &self,
        arguments: EvaluateArguments,
    ) -> io::Result<Response> {
        let expression = ZkEvaluate::try_from(arguments.expression.as_str())?;

        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let result = match expression {
            ZkEvaluate::Constraint { id } => {
                let constraint = debugger.fetch_constraint(id)?;
                let constraint = EncodableConstraint::from(constraint);

                serde_json::to_string(&constraint)?
            }

            ZkEvaluate::CurrentConstraint => {
                let constraint = debugger.fetch_current_constraint()?;
                let constraint = EncodableConstraint::from(constraint);

                serde_json::to_string(&constraint)?
            }

            ZkEvaluate::Witness { id } => {
                let witness = debugger.fetch_witness(id)?;
                let witness = EncodableWitness::from(witness);

                serde_json::to_string(&witness)?
            }
        };

        Ok(Response::Evaluate {
            body: EvaluateResponse {
                result,
                r#type: None,
                presentation_hint: VariablePresentationHint {
                    kind: Kind::Data,
                    attributes: vec![],
                    visibility: None,
                    lazy: false,
                },
                variables_reference: 0,
                named_variables: None,
                indexed_variables: None,
                memory_reference: None,
            },
        })
    }

    async fn initialize(
        &self,
        arguments: InitializeArguments,
    ) -> io::Result<Response> {
        let InitializeArguments { adapter_id, .. } = arguments;

        let path = PathBuf::from(adapter_id);
        let mut debugger = ZkDebugger::open(path)?;

        self.update_constraint(
            &mut debugger,
            StoppedReason::Custom("initialized".into()),
            vec![],
        )
        .await?;

        self.backend.lock().await.replace(debugger);

        Ok(Response::Initialize {
            body: Self::capabilities(),
        })
    }

    async fn r#continue(&self) -> io::Result<Response> {
        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let state = debugger.cont()?;

        self.consume_state(debugger, state).await?;

        Ok(Response::Continue {
            body: ContinueResponse {
                all_threads_continued: true,
            },
        })
    }

    async fn goto(&self, arguments: GotoArguments) -> io::Result<Response> {
        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        debugger.goto(arguments.target_id as usize)?;

        self.update_constraint(debugger, StoppedReason::Goto, vec![])
            .await?;

        Ok(Response::Goto)
    }

    async fn loaded_sources(&self) -> io::Result<Response> {
        let debugger = self.backend.lock().await;
        let debugger = debugger.as_ref().ok_or_else(Self::not_initialized)?;

        // TODO consider using reference instead of path
        let sources = debugger
            .sources()
            .map(|(path, contents)| Source {
                name: None,
                source_reference: Some(SourceReference::Path(path.into())),
                presentation_hint: None,
                origin: Some(contents.into()),
                sources: vec![],
                adapter_data: None,
                checksums: vec![],
            })
            .collect();

        Ok(Response::LoadedSources {
            body: LoadedSourcesResponse { sources },
        })
    }

    async fn next(&self) -> io::Result<Response> {
        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let state = debugger.step()?;

        self.consume_state(debugger, state).await?;

        Ok(Response::Goto)
    }

    async fn restart(&self) -> io::Result<Response> {
        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        debugger.goto(0)?;

        self.events
            .send(Event::Process {
                name: debugger.to_string(),
                system_process_id: None,
                is_local_process: true,
                start_method: Some(ProcessStartMethod::Launch),
                pointer_size: None,
            })
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        self.update_constraint(
            debugger,
            StoppedReason::Custom("restart".into()),
            vec![],
        )
        .await?;

        Ok(Response::Restart)
    }

    async fn reverse_continue(&self) -> io::Result<Response> {
        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let state = debugger.turn()?;

        self.consume_state(debugger, state).await?;

        Ok(Response::Continue {
            body: ContinueResponse {
                all_threads_continued: true,
            },
        })
    }

    async fn add_breakpoint(
        &self,
        arguments: CustomAddBreakpointArguments,
    ) -> io::Result<Response> {
        let CustomAddBreakpointArguments { breakpoint } = arguments;

        let line = breakpoint.line;
        let name = breakpoint.source.and_then(|s| s.name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "the breakpoint name wasn't provided",
            )
        })?;

        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let id = debugger.add_breakpoint(name, line) as u64;

        Ok(Response::CustomAddBreakpoint {
            body: CustomAddBreakpointResponse { id },
        })
    }

    async fn remove_breakpoint(
        &self,
        arguments: CustomRemoveBreakpointArguments,
    ) -> io::Result<Response> {
        let CustomRemoveBreakpointArguments { id } = arguments;

        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let removed = debugger.remove_breakpoint(id as usize).is_some();

        Ok(Response::CustomRemoveBreakpoint {
            body: CustomRemoveBreakpointResponse { id, removed },
        })
    }

    async fn set_breakpoints(
        &self,
        arguments: SetBreakpointsArguments,
    ) -> io::Result<Response> {
        let SetBreakpointsArguments {
            source,
            breakpoints,
            lines,
            ..
        } = arguments;

        let path = match source.source_reference {
            Some(SourceReference::Path(path)) => path,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "the source path if mandatory to set a breakpoint",
                ))
            }
        };

        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        debugger.clear_breakpoints(path.as_str());

        let breakpoints = breakpoints
            .into_iter()
            .map(|b| b.line)
            .chain(lines.into_iter())
            .map(|line| {
                let id = debugger.add_breakpoint(path.clone(), Some(line));

                Breakpoint {
                    id: Some(id as u64),
                    verified: true,
                    message: None,
                    source: None,
                    line: Some(line),
                    column: None,
                    end_line: Some(line),
                    end_column: None,
                    instruction_reference: None,
                    offset: None,
                }
            })
            .collect();

        Ok(Response::SetBreakpoints {
            body: SetBreakpointsResponse { breakpoints },
        })
    }

    async fn step_back(&self) -> io::Result<Response> {
        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let state = debugger.afore()?;

        self.consume_state(debugger, state).await?;

        Ok(Response::Goto)
    }
}

#[async_trait]
impl Backend for ZkDap {
    async fn init(
        events: Sender<Event>,
        _requests: Sender<ReactorReverseRequest>,
    ) -> Self {
        let backend = None;
        let backend = Mutex::new(backend);
        let backend = Arc::new(backend);

        ZkDap { events, backend }
    }

    async fn request(&mut self, request: Request) -> Option<Response> {
        tracing::debug!("request received: {:?}", request);

        let response = match request {
            // attach won't affect the state of the dap - we can have many
            // clients attached
            Request::Attach { .. } => Ok(Some(Response::Attach)),

            Request::BreakpointLocations { arguments } => {
                self.breakpoint_locations(arguments).await.map(Some)
            }

            // the backend is immediately ready after load
            Request::ConfigurationDone { .. } => {
                Ok(Some(Response::ConfigurationDone))
            }

            Request::Continue { .. } => self.r#continue().await.map(Some),

            Request::CustomAddBreakpoint { arguments } => {
                self.add_breakpoint(arguments).await.map(Some)
            }

            Request::CustomRemoveBreakpoint { arguments } => {
                self.remove_breakpoint(arguments).await.map(Some)
            }

            // we might implement multi-session per dap provider in the future
            Request::Disconnect { .. } => Ok(Some(Response::Disconnect)),
            Request::Terminate { .. } => Ok(Some(Response::Terminate)),
            Request::Launch { .. } => Ok(Some(Response::Launch)),

            Request::Evaluate { arguments } => {
                self.evaluate(arguments).await.map(Some)
            }

            Request::Goto { arguments } => self.goto(arguments).await.map(Some),

            Request::Initialize { arguments } => {
                self.initialize(arguments).await.map(Some)
            }

            Request::LoadedSources { .. } => {
                self.loaded_sources().await.map(Some)
            }

            Request::Next { .. } => self.next().await.map(Some),

            Request::Restart { .. } => self.restart().await.map(Some),

            Request::ReverseContinue { .. } => {
                self.reverse_continue().await.map(Some)
            }

            Request::SetBreakpoints { arguments } => {
                self.set_breakpoints(arguments).await.map(Some)
            }

            Request::StepBack { .. } => self.step_back().await.map(Some),

            _ => {
                tracing::warn!("not implemented");
                Ok(None)
            }
        };

        response
            .map(|response| {
                tracing::debug!("responding {:?}", response);
                response
            })
            .unwrap_or_else(|e| {
                tracing::warn!("error responding request: {}", e);

                self.events
                    .try_send(Event::Output {
                        category: Some(OutputCategory::Stderr),
                        output: e.to_string(),
                        group: None,
                        variables_reference: 0,
                        source: None,
                        line: None,
                        column: None,
                        data: None,
                    })
                    .ok();

                None
            })
    }

    async fn response(&mut self, _id: u64, response: Response) {
        tracing::debug!("reverse requests are not applicable: {:?}", response);
    }
}
