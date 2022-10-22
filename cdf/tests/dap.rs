use std::path::PathBuf;
use std::{env, io};

use dap_reactor::prelude::*;
use dusk_cdf::*;

#[tokio::test]
async fn initialize_works() -> io::Result<()> {
    let service = dusk_cdf::ZkDapBuilder::new("127.0.0.1:0").build().await?;

    let socket = service.local_addr()?;

    tokio::spawn(async move {
        service.listen().await.ok();
    });

    let cdf = env!("CARGO_MANIFEST_DIR");
    let cdf = PathBuf::from(cdf)
        .parent()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "failed to find project root dir",
            )
        })?
        .join("assets")
        .join("test.cdf")
        .canonicalize()?
        .display()
        .to_string();

    let mut client = ClientBuilder::new().connect(socket).await?;

    client
        .requests
        .send(
            Request::Initialize {
                arguments: InitializeArguments {
                    client_id: None,
                    client_name: None,
                    adapter_id: "cdf".into(),
                    locale: None,
                    lines_start_at_1: true,
                    column_start_at_1: true,
                    path_format: None,
                    supports_variable_type: true,
                    supports_variable_paging: false,
                    supports_run_in_terminal_request: false,
                    supports_memory_references: false,
                    supports_progress_reporting: false,
                    supports_invalidated_event: false,
                    supports_memory_event: false,
                    supports_args_can_be_interpreted_by_shell: false,
                },
            }
            .into(),
        )
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    client
        .requests
        .send(Request::from(ZkRequest::LoadCdf { path: cdf }).into())
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    client
        .requests
        .send(Request::Restart { arguments: None }.into())
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    client
        .requests
        .send(
            Request::Continue {
                arguments: ContinueArguments {
                    thread_id: 0,
                    single_thread: true,
                },
            }
            .into(),
        )
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    client
        .requests
        .send(
            Request::ReverseContinue {
                arguments: ReverseContinueArguments {
                    thread_id: 0,
                    single_thread: true,
                },
            }
            .into(),
        )
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    client
        .requests
        .send(Request::Next { arguments: None }.into())
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    client
        .requests
        .send(
            Request::StepBack {
                arguments: StepBackArguments {
                    thread_id: 0,
                    single_thread: true,
                    granularity: None,
                },
            }
            .into(),
        )
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    client
        .requests
        .send(
            Request::StepBack {
                arguments: StepBackArguments {
                    thread_id: 0,
                    single_thread: true,
                    granularity: None,
                },
            }
            .into(),
        )
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    client
        .requests
        .send(
            Request::from(ZkRequest::AddBreakpoint {
                breakpoint: dap_reactor::prelude::Breakpoint {
                    id: None,
                    verified: true,
                    message: None,
                    source: Some(Source {
                        name: Some(String::from("hash")),
                        source_reference: None,
                        presentation_hint: None,
                        origin: None,
                        sources: vec![],
                        adapter_data: None,
                        checksums: vec![],
                    }),
                    line: Some(5),
                    column: None,
                    end_line: Some(5),
                    end_column: None,
                    instruction_reference: None,
                    offset: None,
                },
            })
            .into(),
        )
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    client
        .requests
        .send(Request::from(ZkRequest::RemoveBreakpoint { id: 1 }).into())
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let ClientResponse { response, .. } =
        client.responses.recv().await.ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "expected response")
        })?;

    let capabilities = match response {
        Response::Initialize { body } => Ok(body),
        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            "wrong response variant",
        )),
    }?;

    assert_eq!(ZkDap::capabilities(), capabilities);

    while client.responses.try_recv().is_ok() {}

    Ok(())
}

#[test]
fn request_encode_decode() {
    // assert won't panic
    assert!(ZkRequest::try_from(None).is_err());

    fn run(request: ZkRequest) {
        let client = ClientRequest::from(request.clone());
        let value = match client.request {
            Request::Custom {
                arguments: Some(value),
            } => value,

            _ => panic!("failed to fetch custom request"),
        };

        let r = ZkRequest::try_from(Some(&value))
            .expect("failed to reconstruct request");

        assert_eq!(request, r);
    }

    let cases = vec![
        ZkRequest::AddBreakpoint {
            breakpoint: dap_reactor::prelude::Breakpoint {
                id: Some(15),
                verified: true,
                message: Some("foo".into()),
                source: None,
                line: Some(20),
                column: None,
                end_line: None,
                end_column: None,
                instruction_reference: None,
                offset: None,
            },
        },
        ZkRequest::RemoveBreakpoint { id: 48 },
        ZkRequest::LoadCdf { path: "foo".into() },
        ZkRequest::SourceContents,
        ZkRequest::Witness { id: 38 },
    ];

    for case in cases {
        run(case);
    }
}

#[test]
fn response_encode_decode() {
    // assert won't panic
    assert!(ZkResponse::try_from(None).is_err());

    fn run(response: ZkResponse) {
        let value = match Response::from(response.clone()) {
            Response::Custom { body: Some(value) } => value,

            _ => panic!("failed to fetch custom response"),
        };

        let r = ZkResponse::try_from(Some(&value))
            .expect("failed to reconstruct response");

        assert_eq!(response, r);
    }

    let cases = vec![
        ZkResponse::AddBreakpoint { id: 38 },
        ZkResponse::RemoveBreakpoint {
            id: 92,
            removed: true,
        },
        ZkResponse::LoadCdf,
        ZkResponse::SourceContents {
            sources: vec![ZkSource {
                path: "foo".into(),
                contents: "bar".into(),
            }],
        },
        ZkResponse::Witness {
            witness: ZkWitness {
                id: 92,
                constraint: Some(28),
                value: "foo".into(),
                source: "bar".into(),
                line: 19,
            },
        },
    ];

    for case in cases {
        run(case);
    }
}
