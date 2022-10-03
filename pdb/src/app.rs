mod config;
mod input;
mod output;

use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use crate::args::ParsedArgs;
use crate::commands::Command;
use dap_reactor::prelude::{
    CustomAddBreakpointResponse, CustomRemoveBreakpointResponse,
    EvaluateResponse, Event, LoadedSourcesResponse, Source as DapSource,
};
use dap_reactor::protocol::ProtocolResponseError;
use dap_reactor::reactor::{
    Client, ClientBuilder, ClientRequest, ClientResponse,
};
use dap_reactor::request::Request;
use dap_reactor::response::Response;
use dap_reactor::types::SourceReference;
use dusk_cdf::{BaseConfig, ZkSourceDescription};
use tokio::sync::{mpsc, RwLock};

use config::Config;
use input::Input;

pub use output::{Output, Source};

pub struct App {
    config: Config,
    input: Input,
    requests: mpsc::Sender<ClientRequest>,
    outputs: mpsc::Receiver<Output>,
}

impl App {
    pub const fn config(&self) -> &Config {
        &self.config
    }

    async fn handle_events(
        contents: Arc<RwLock<HashMap<String, String>>>,
        mut events: mpsc::Receiver<Event>,
        outputs: mpsc::Sender<Output>,
    ) {
        while let Some(event) = events.recv().await {
            if let Event::Stopped {
                description: Some(d),
                ..
            } = event
            {
                if let Ok(ZkSourceDescription { name, line }) =
                    ZkSourceDescription::try_from(d.as_str())
                {
                    if let Some(contents) =
                        contents.read().await.get(&name).cloned()
                    {
                        outputs
                            .send(Output {
                                contents: Some(Source {
                                    name,
                                    contents,
                                    line: line as usize,
                                }),
                                console: vec![],
                                error: vec![],
                            })
                            .await
                            .ok();
                    }
                }
            }
        }
    }

    async fn handle_responses(
        contents: Arc<RwLock<HashMap<String, String>>>,
        requests: mpsc::Sender<ClientRequest>,
        mut responses: mpsc::Receiver<ClientResponse>,
        outputs: mpsc::Sender<Output>,
    ) {
        while let Some(ClientResponse { response, .. }) = responses.recv().await
        {
            match response {
                Response::Error {
                    command,
                    error: ProtocolResponseError { message, .. },
                } => {
                    let mut error = format!("error in command '{}'", command);

                    if let Some(m) = message {
                        error.push_str(format!(": {}", m).as_str());
                    }

                    outputs
                        .send(Output {
                            contents: None,
                            console: vec![],
                            error: vec![error],
                        })
                        .await
                        .ok();
                }

                Response::Evaluate {
                    body: EvaluateResponse { result, .. },
                } => {
                    // TODO pretty print the eval result
                    outputs
                        .send(Output {
                            contents: None,
                            console: vec![result],
                            error: vec![],
                        })
                        .await
                        .ok();
                }

                Response::Initialize { .. } => {
                    requests
                        .send(ClientRequest {
                            seq: None,
                            request: Request::LoadedSources { arguments: None },
                        })
                        .await
                        .ok();
                }

                Response::LoadedSources {
                    body: LoadedSourcesResponse { sources },
                } => {
                    let mut contents = contents.write().await;

                    contents.clear();

                    let sources = sources.into_iter().filter_map(
                        |DapSource {
                             source_reference,
                             origin,
                             ..
                         }| match (
                            source_reference,
                            origin,
                        ) {
                            (
                                Some(SourceReference::Path(name)),
                                Some(contents),
                            ) => Some((name, contents)),
                            _ => None,
                        },
                    );

                    contents.extend(sources);

                    outputs
                        .send(Output {
                            contents: None,
                            console: vec!["file loaded!".into()],
                            error: vec![],
                        })
                        .await
                        .ok();
                }

                Response::CustomAddBreakpoint {
                    body: CustomAddBreakpointResponse { id },
                } => {
                    outputs
                        .send(Output {
                            contents: None,
                            console: vec![format!("breakpoint added: #{}", id)],
                            error: vec![],
                        })
                        .await
                        .ok();
                }

                Response::CustomRemoveBreakpoint {
                    body: CustomRemoveBreakpointResponse { id, removed },
                } => {
                    outputs
                        .send(Output {
                            contents: None,
                            console: removed
                                .then(|| {
                                    vec![format!("breakpoint #{} removed", id)]
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
                        .await
                        .ok();
                }

                // the concrete implementation use `Stopped` event as output
                // signal
                _ => (),
            }
        }
    }

    pub async fn load(args: ParsedArgs) -> io::Result<Self> {
        let ParsedArgs { path, socket } = args;
        let config = Config::load()?;

        let input = Input::try_from(&config)?;

        let service = dusk_cdf::ZkDapBuilder::new(socket).build().await?;
        let socket = service.local_addr()?;

        let contents: HashMap<String, String> = HashMap::new();
        let contents = RwLock::new(contents);
        let contents = Arc::new(contents);

        tokio::spawn(async move {
            service.listen().await.ok();
        });

        let Client {
            responses,
            events,
            requests,
            ..
        } = ClientBuilder::new().connect(socket).await?;

        if let Some(path) = path {
            let command = Command::Open { path };

            if let Some(request) = command.request().first().cloned() {
                requests
                    .send(request.into())
                    .await
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            }
        }

        let (outputs_tx, outputs) = mpsc::channel(50);

        let o = outputs_tx.clone();
        let c = Arc::clone(&contents);

        tokio::spawn(async move {
            Self::handle_events(c, events, o).await;
        });

        let o = outputs_tx;
        let c = Arc::clone(&contents);
        let r = requests.clone();

        tokio::spawn(async move {
            Self::handle_responses(c, r, responses, o).await;
        });

        Ok(Self {
            config,
            input,
            requests,
            outputs,
        })
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

        let requests = command.request();
        if requests.is_empty() {
            return Some(Output {
                contents: None,
                console: vec![],
                error: vec!["the provided command didn't translate into a valid request".into()],
            });
        }

        for request in requests {
            if let Err(e) = self.requests.send(request.into()).await {
                return Some(Output {
                    contents: None,
                    console: vec![],
                    error: vec![format!(
                        "error sending request to backend: {}",
                        e
                    )],
                });
            }
        }

        let mut output = match self.outputs.recv().await {
            Some(o) => o,

            None => {
                return Some(Output {
                    contents: None,
                    console: vec![],
                    error: vec![format!(
                        "the outputs channel is in an invalid state!"
                    )],
                })
            }
        };

        while let Ok(o) = self.outputs.try_recv() {
            output.merge(o);
        }

        Some(output)
    }
}
