//! Debug Adapter Protocol provider

mod types;
mod utils;

#[cfg(test)]
mod tests;

use std::fs::File;
use std::io;
use std::net::SocketAddr;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

use dap_reactor::prelude::*;
use tokio::net;
use tokio::sync::Mutex;

use crate::{State, ZkDebugger};

pub use types::*;

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
    pub const fn capabilities() -> Capabilities {
        Capabilities {
            supports_configuration_done_request: true,
            supports_function_breakpoints: true,
            supports_conditional_breakpoints: true,
            supports_hit_conditional_breakpoints: true,
            supports_evaluate_for_hovers: false,
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
            supported_checksum_algorithms: vec![],
            supports_restart_request: true,
            supports_exception_options: false,
            supports_value_formatting_options: false,
            supports_exception_info_request: false,
            support_terminate_debuggee: false,
            support_suspend_debuggee: false,
            supports_delayed_stack_trace_loading: false,
            supports_loaded_sources_request: false,
            supports_log_points: false,
            supports_terminate_threads_request: false,
            supports_set_expression: false,
            supports_terminate_request: false,
            supports_data_breakpoints: true,
            supports_read_memory_request: false,
            supports_write_memory_request: false,
            supports_disassemble_request: false,
            supports_cancel_request: false,
            supports_breakpoint_locations_request: true,
            supports_clipboard_context: false,
            supports_stepping_granularity: false,
            supports_instruction_breakpoints: true,
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

    async fn send_event(&self, event: Event) -> io::Result<()> {
        self.events
            .send(event)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    async fn update_constraint(
        &self,
        reason: StoppedReason,
        breakpoints: Vec<usize>,
    ) -> io::Result<()> {
        self.send_event(Event::Stopped {
            reason,
            description: None,
            thread_id: Some(0),
            preserve_focus_hint: false,
            text: None,
            all_threads_stopped: true,
            hit_breakpoint_ids: breakpoints,
        })
        .await
    }

    async fn terminate(&self, exit_code: u64) -> io::Result<()> {
        self.send_event(Event::Thread {
            reason: ThreadReason::Exited,
            thread_id: 0,
        })
        .await?;

        self.send_event(Event::Terminated { restart: None }).await?;
        self.send_event(Event::Exited { exit_code }).await?;

        Ok(())
    }

    async fn consume_state(&self, state: State) -> io::Result<()> {
        match state {
            State::Beginning | State::Constraint { .. } => {
                self.update_constraint(StoppedReason::Step, vec![]).await?;
            }

            State::InvalidConstraint { .. } => {
                self.terminate(1).await?;
            }

            State::Breakpoint { id } => {
                self.update_constraint(StoppedReason::Breakpoint, vec![id])
                    .await?;
            }

            State::End { .. } => {
                self.terminate(0).await?;
            }
        }

        Ok(())
    }

    async fn breakpoint_locations(
        &self,
        arguments: Option<BreakpointLocationsArguments>,
    ) -> io::Result<Response> {
        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let (source, line, end_line) = match arguments {
            Some(BreakpointLocationsArguments {
                source:
                    Source {
                        source_reference: Some(SourceReference::Path(path)),
                        ..
                    },
                line,
                end_line,
                ..
            }) => (path, line, end_line),
            _ => return Ok(Response::BreakpointLocations { body: None }),
        };

        let end_line = end_line.unwrap_or(line);
        let breakpoints = (line..=end_line)
            .filter(|l| debugger.add_breakpoint(source.clone(), Some(*l)) > 0)
            .map(|_| BreakpointLocation {
                line,
                column: None,
                end_line: None,
                end_column: None,
            })
            .collect();

        Ok(Response::BreakpointLocations {
            body: Some(BreakpointLocationsResponse { breakpoints }),
        })
    }

    async fn evaluate(&self) -> io::Result<Response> {
        Ok(Response::Evaluate {
            body: EvaluateResponse {
                result: "evaluate is not implemented".into(),
                r#type: None,
                presentation_hint: Some(VariablePresentationHint {
                    kind: Some(VariablePresentationHintKind::Data),
                    attributes: vec![],
                    visibility: None,
                    lazy: false,
                }),
                variables_reference: 0,
                named_variables: None,
                indexed_variables: None,
                memory_reference: None,
            },
        })
    }

    async fn initialize(&self) -> io::Result<Response> {
        self.send_event(Event::Initialized).await?;

        Ok(Response::Initialize {
            body: Self::capabilities(),
        })
    }

    async fn r#continue(&self) -> io::Result<Response> {
        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let state = debugger.cont()?;

        self.consume_state(state).await?;

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

        self.update_constraint(StoppedReason::Goto, vec![]).await?;

        Ok(Response::Goto)
    }

    async fn next(&self) -> io::Result<Response> {
        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let state = debugger.step()?;

        self.consume_state(state).await?;

        Ok(Response::Goto)
    }

    async fn restart(&self) -> io::Result<Response> {
        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        debugger.goto(0)?;

        self.send_event(Event::Process {
            name: debugger.to_string(),
            system_process_id: None,
            is_local_process: true,
            start_method: Some(ProcessStartMethod::Launch),
            pointer_size: None,
        })
        .await?;

        self.update_constraint(StoppedReason::Step, vec![]).await?;

        Ok(Response::Restart)
    }

    async fn reverse_continue(&self) -> io::Result<Response> {
        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let state = debugger.turn()?;

        self.consume_state(state).await?;

        Ok(Response::Continue {
            body: ContinueResponse {
                all_threads_continued: true,
            },
        })
    }

    async fn custom_request(
        &self,
        arguments: Option<Value>,
    ) -> io::Result<Response> {
        let request = ZkRequest::try_from(arguments.as_ref())?;

        match request {
            ZkRequest::AddBreakpoint { breakpoint } => {
                self.add_breakpoint(breakpoint).await
            }

            ZkRequest::RemoveBreakpoint { id } => {
                self.remove_breakpoint(id).await
            }

            ZkRequest::LoadCdf { path } => self.load_cdf(path).await,

            ZkRequest::SourceContents => self.source_contents().await,

            ZkRequest::Witness { id } => self.witness(id).await,
        }
    }

    async fn add_breakpoint(
        &self,
        breakpoint: Breakpoint,
    ) -> io::Result<Response> {
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

        Ok(ZkResponse::AddBreakpoint { id }.into())
    }

    async fn remove_breakpoint(&self, id: u64) -> io::Result<Response> {
        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let removed = debugger.remove_breakpoint(id as usize).is_some();

        Ok(ZkResponse::RemoveBreakpoint { id, removed }.into())
    }

    async fn load_cdf(&self, path: String) -> io::Result<Response> {
        let path = PathBuf::from(path);
        let debugger = ZkDebugger::open(path)?;

        self.send_event(Event::Thread {
            reason: ThreadReason::Started,
            thread_id: 0,
        })
        .await?;

        self.update_constraint(StoppedReason::Step, vec![]).await?;

        self.backend.lock().await.replace(debugger);

        Ok(ZkResponse::LoadCdf.into())
    }

    async fn source_contents(&self) -> io::Result<Response> {
        let debugger = self.backend.lock().await;
        let debugger = debugger.as_ref().ok_or_else(Self::not_initialized)?;

        let sources = debugger
            .sources()
            .map(|(path, contents)| ZkSource {
                path: path.into(),
                contents: contents.into(),
            })
            .collect();

        Ok(ZkResponse::SourceContents { sources }.into())
    }

    async fn scopes(&self) -> io::Result<Response> {
        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let constraint = debugger.fetch_current_constraint()?;
        let variables_reference = constraint.id() as u64;
        let source = Source::from(&constraint);
        let line = constraint.line();
        let column = constraint.col();

        Ok(Response::Scopes {
            body: ScopesResponse {
                scopes: vec![Scope {
                    name: "Circuit".into(),
                    presentation_hint: Some(ScopePresentationHint::Locals),
                    variables_reference,
                    named_variables: Some(18),
                    indexed_variables: Some(18),
                    expensive: false,
                    source: Some(source),
                    line: Some(line),
                    column: Some(column),
                    end_line: None,
                    end_column: None,
                }],
            },
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

    async fn stack_trace(&self) -> io::Result<Response> {
        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let constraint = debugger.fetch_current_constraint()?;
        let source = Source::from(&constraint);

        let line = constraint.line();
        let column = constraint.col();

        Ok(Response::StackTrace {
            body: StackTraceResponse {
                stack_frames: vec![StackFrame {
                    id: 0,
                    name: "cdf".into(),
                    source: Some(source),
                    line,
                    column,
                    end_line: None,
                    end_column: None,
                    can_restart: true,
                    instruction_pointer_reference: None,
                    module_id: None,
                    presentation_hint: None,
                }],
                total_frames: Some(1),
            },
        })
    }

    async fn step_back(&self) -> io::Result<Response> {
        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let state = debugger.afore()?;

        self.consume_state(state).await?;

        Ok(Response::Goto)
    }

    async fn threads(&self) -> io::Result<Response> {
        Ok(Response::Threads {
            body: ThreadsResponse {
                threads: vec![Thread {
                    id: 0,
                    name: "cdf".into(),
                }],
            },
        })
    }

    async fn variables(
        &self,
        arguments: VariablesArguments,
    ) -> io::Result<Response> {
        if let Some(VariablesArgumentsFilter::Named) = arguments.filter.as_ref()
        {
            return Ok(Response::Variables {
                body: VariablesResponse { variables: vec![] },
            });
        }

        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let constraint = debugger.fetch_current_constraint()?;
        let id = constraint.id();

        let polynomial = *constraint.polynomial();

        let idx = utils::idx_to_var("constraint", id);

        let qm = utils::scalar_to_var("Qm", &polynomial.selectors.qm);
        let ql = utils::scalar_to_var("Ql", &polynomial.selectors.ql);
        let qr = utils::scalar_to_var("Qr", &polynomial.selectors.qr);
        let qd = utils::scalar_to_var("Qd", &polynomial.selectors.qd);
        let qc = utils::scalar_to_var("Qc", &polynomial.selectors.qc);
        let qo = utils::scalar_to_var("Qo", &polynomial.selectors.qo);
        let pi = utils::scalar_to_var("PI", &polynomial.selectors.pi);
        let qarith =
            utils::scalar_to_var("Qarith", &polynomial.selectors.qarith);
        let qlogic =
            utils::scalar_to_var("Qlogic", &polynomial.selectors.qlogic);
        let qrange =
            utils::scalar_to_var("Qrange", &polynomial.selectors.qrange);
        let qgroup = utils::scalar_to_var(
            "Qgroup",
            &polynomial.selectors.qgroup_variable,
        );
        let qadd =
            utils::scalar_to_var("Qadd", &polynomial.selectors.qfixed_add);

        let eval = utils::bool_to_var("Evaluation", polynomial.evaluation);

        let wa = debugger
            .fetch_witness(polynomial.witnesses.a)
            .map(|w| utils::witness_to_var("Wa", w))?;
        let wb = debugger
            .fetch_witness(polynomial.witnesses.b)
            .map(|w| utils::witness_to_var("Wb", w))?;
        let wd = debugger
            .fetch_witness(polynomial.witnesses.d)
            .map(|w| utils::witness_to_var("Wd", w))?;
        let wo = debugger
            .fetch_witness(polynomial.witnesses.o)
            .map(|w| utils::witness_to_var("Wo", w))?;

        Ok(Response::Variables {
            body: VariablesResponse {
                variables: vec![
                    idx, qm, ql, qr, qd, qc, qo, pi, qarith, qlogic, qrange,
                    qgroup, qadd, eval, wa, wb, wd, wo,
                ],
            },
        })
    }

    async fn witness(&self, id: usize) -> io::Result<Response> {
        let mut debugger = self.backend.lock().await;
        let debugger = debugger.as_mut().ok_or_else(Self::not_initialized)?;

        let witness = debugger.fetch_witness(id)?;
        let witness = ZkWitness::from(witness);

        Ok(ZkResponse::Witness { witness }.into())
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

            Request::Custom { arguments } => {
                self.custom_request(arguments).await.map(Some)
            }

            // we might implement multi-session per dap provider in the future
            Request::Disconnect { .. } => Ok(Some(Response::Disconnect)),
            Request::Terminate { .. } => Ok(Some(Response::Terminate)),
            Request::Launch { .. } => Ok(Some(Response::Launch)),

            Request::Evaluate { .. } => self.evaluate().await.map(Some),

            Request::Goto { arguments } => self.goto(arguments).await.map(Some),

            Request::Initialize { .. } => self.initialize().await.map(Some),

            Request::Next { .. } => self.next().await.map(Some),

            Request::Restart { .. } => self.restart().await.map(Some),

            Request::ReverseContinue { .. } => {
                self.reverse_continue().await.map(Some)
            }

            Request::Scopes { .. } => self.scopes().await.map(Some),

            Request::SetBreakpoints { arguments } => {
                self.set_breakpoints(arguments).await.map(Some)
            }

            Request::StackTrace { .. } => self.stack_trace().await.map(Some),

            Request::StepBack { .. } => self.step_back().await.map(Some),

            Request::Threads => self.threads().await.map(Some),

            Request::Variables { arguments } => {
                self.variables(arguments).await.map(Some)
            }

            _ => {
                tracing::warn!("not supported");
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
                        variables_reference: None,
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
