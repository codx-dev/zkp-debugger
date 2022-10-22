mod config;
mod context;
mod input;
mod output;

use std::{io, net};

use crate::args::ParsedArgs;
use crate::commands::Command;
use dap_reactor::prelude::{
    Event, Source as DapSource, StackTraceArguments, StackTraceResponse,
    ThreadReason, VariablesResponse,
};
use dap_reactor::prelude::{SourceReference, StackFrame};
use dap_reactor::protocol::ProtocolResponseError;
use dap_reactor::reactor::{Client, ClientBuilder, ClientResponse};
use dap_reactor::request::Request;
use dap_reactor::response::Response;
use dusk_cdf::{ZkRequest, ZkResponse};
use tokio::sync::mpsc;
use tokio::time;
use toml_base_config::BaseConfig;

use config::Config;
use context::Context;
use input::Input;

pub use output::{Output, Source};

pub struct App {
    context: Context,
    input: Input,
    outputs: mpsc::Receiver<Output>,
}

impl App {
    pub const fn config(&self) -> &Config {
        self.context.config()
    }

    async fn handle_events(
        context: Context,
        mut events: mpsc::Receiver<Event>,
    ) {
        while let Some(event) = events.recv().await {
            let mut result = None;

            match event {
                Event::Initialized => {
                    if let Some(path) = context.path().await {
                        context.lock_contents().await;

                        result.replace(
                            context
                                .send_request(ZkRequest::LoadCdf { path })
                                .await,
                        );
                    }
                }

                Event::Thread {
                    reason: ThreadReason::Started,
                    ..
                } => {
                    result.replace(
                        context.send_request(ZkRequest::SourceContents).await,
                    );
                }

                Event::Stopped { thread_id, .. } => {
                    result.replace(
                        context
                            .send_request(Request::StackTrace {
                                arguments: StackTraceArguments {
                                    thread_id: thread_id.unwrap_or(0),
                                    start_frame: None,
                                    levels: None,
                                    format: None,
                                },
                            })
                            .await,
                    );
                }

                Event::Thread {
                    reason: ThreadReason::Exited,
                    ..
                } => {
                    result.replace(
                        context
                            .send_output(Output {
                                contents: None,
                                console: vec!["execution finished".into()],
                                error: vec![],
                            })
                            .await,
                    );
                }

                _ => (),
            }

            if let Some(Err(e)) = result {
                context.send_error_output(e).await;
            }
        }
    }

    async fn handle_responses(
        context: Context,
        mut responses: mpsc::Receiver<ClientResponse>,
    ) {
        while let Some(ClientResponse { response, .. }) = responses.recv().await
        {
            let mut result: Option<io::Result<()>> = None;
            let mut custom: Option<ZkResponse> = None;

            match response {
                Response::Custom { body } => {
                    ZkResponse::try_from(body.as_ref())
                        .map(|r| custom.replace(r))
                        .ok();
                }

                Response::Error {
                    command,
                    error: ProtocolResponseError { message, .. },
                } => {
                    let mut error = format!("error in command '{}'", command);

                    if let Some(m) = message {
                        error.push_str(format!(": {}", m).as_str());
                    }

                    context.send_error_output(error).await;
                }

                Response::StackTrace {
                    body: StackTraceResponse { stack_frames, .. },
                } => {
                    if let Some(StackFrame {
                        source:
                            Some(DapSource {
                                source_reference:
                                    Some(SourceReference::Path(path)),
                                ..
                            }),
                        line,
                        ..
                    }) = stack_frames.into_iter().next()
                    {
                        if let Some(contents) = context.contents(&path).await {
                            let output = Output {
                                contents: Some(Source {
                                    name: path,
                                    contents,
                                    line: line as usize,
                                }),
                                console: vec![],
                                error: vec![],
                            };

                            result.replace(context.send_output(output).await);
                        }
                    }
                }

                Response::Variables {
                    body: VariablesResponse { variables },
                } => {
                    let mut output = Output::default();

                    for v in variables {
                        output.merge(Output {
                            contents: None,
                            console: vec![format!("{}: {}", v.name, v.value)],
                            error: vec![],
                        });
                    }

                    result.replace(context.send_output(output).await);
                }

                _ => (),
            }

            match custom {
                Some(ZkResponse::SourceContents { sources }) => {
                    context.replace_contents_batch(sources).await;
                    context.unlock_contents().await;
                }

                Some(ZkResponse::AddBreakpoint { id }) => {
                    result.replace(
                        context
                            .send_output(Output {
                                contents: None,
                                console: vec![format!(
                                    "breakpoint added: #{}",
                                    id
                                )],
                                error: vec![],
                            })
                            .await,
                    );
                }

                Some(ZkResponse::RemoveBreakpoint { id, removed }) => {
                    result.replace(
                        context
                            .send_output(Output {
                                contents: None,
                                console: removed
                                    .then(|| {
                                        vec![format!(
                                            "breakpoint #{} removed",
                                            id
                                        )]
                                    })
                                    .unwrap_or_default(),
                                error: (!removed)
                                    .then(|| {
                                        vec![format!(
                                            "breakpoint #{} wasn't removed!",
                                            id
                                        )]
                                    })
                                    .unwrap_or_default(),
                            })
                            .await,
                    );
                }

                Some(ZkResponse::Witness { witness }) => {
                    result.replace(
                        context
                            .send_output(Output {
                                contents: None,
                                console: vec![format!("{:?}", witness)],
                                error: vec![],
                            })
                            .await,
                    );
                }

                _ => (),
            }

            if let Some(Err(e)) = result {
                context.send_error_output(e).await;
            }
        }
    }

    pub async fn load(args: ParsedArgs) -> io::Result<Self> {
        let ParsedArgs { path, attach } = args;
        let config = Config::load()?;

        let input = Input::try_from(&config)?;

        let socket = match attach {
            Some(socket) => socket,

            None => {
                let ip = net::Ipv4Addr::LOCALHOST;
                let port = 0;
                let socket = net::SocketAddrV4::new(ip, port);

                let service =
                    dusk_cdf::ZkDapBuilder::new(socket).build().await?;
                let socket = service.local_addr()?;

                tokio::spawn(async move {
                    service.listen().await.ok();
                });

                socket
            }
        };

        let Client {
            responses,
            events,
            requests,
            ..
        } = ClientBuilder::new().connect(socket).await?;

        let (outputs_tx, outputs) = mpsc::channel(50);

        let context = Context::new(config, requests, outputs_tx);

        if let Some(path) = path {
            context.replace_path(path.display().to_string()).await?;
        }

        let c = context.clone();

        tokio::spawn(async move {
            Self::handle_events(c, events).await;
        });

        let c = context.clone();

        tokio::spawn(async move {
            Self::handle_responses(c, responses).await;
        });

        let app = Self {
            context,
            input,
            outputs,
        };

        Ok(app)
    }

    /// Empty the pending outputs
    pub async fn flush_output(&mut self) -> Option<Output> {
        time::sleep(self.context.config().render_delay()).await;

        let mut output = Output::default();

        while let Ok(o) = self.outputs.try_recv() {
            output.merge(o);
        }

        Some(output)
    }

    /// Analogous to iterator next, but async
    pub async fn next_output(&mut self) -> Option<Output> {
        let command = match self.input.next() {
            Some(Command::Quit) | None => return None,
            Some(c) => c,
        };

        if matches!(command, Command::Help) {
            return Some(Output {
                contents: None,
                console: vec![self.input.help()],
                error: vec![],
            });
        }

        if let Err(e) = self.context.receive_command(command).await {
            return Some(Output {
                contents: None,
                console: vec![],
                error: vec![format!("error sending request to backend: {}", e)],
            });
        }

        self.flush_output().await
    }
}
