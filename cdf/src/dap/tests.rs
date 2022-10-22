use std::sync::Arc;

use dap_reactor::models::Source;
use tokio::sync::{mpsc, Mutex};

use super::*;

impl From<ZkRequest> for Value {
    fn from(req: ZkRequest) -> Self {
        match Request::from(req) {
            Request::Custom { arguments: Some(v) } => v,
            _ => panic!("failed to generate value"),
        }
    }
}

impl From<Response> for ZkResponse {
    fn from(r: Response) -> Self {
        match r {
            Response::Custom { body } => ZkResponse::try_from(body.as_ref())
                .expect("failed to get response"),
            _ => panic!("unexpected response"),
        }
    }
}

#[tokio::test]
async fn bind_client_wont_panic() -> io::Result<()> {
    let dap = ZkDapBuilder::new("127.0.0.1:0").build().await?;
    let socket = dap.local_addr()?;

    tokio::spawn(async move {
        dap.listen().await.ok();
    });

    let mut client = dap_reactor::reactor::ClientBuilder::new()
        .connect(socket)
        .await?;

    client
        .request(Request::Initialize {
            arguments: InitializeArguments {
                client_id: None,
                client_name: None,
                adapter_id: "cdf".into(),
                locale: None,
                lines_start_at_1: true,
                column_start_at_1: true,
                path_format: None,
                supports_variable_type: true,
                supports_variable_paging: true,
                supports_run_in_terminal_request: true,
                supports_memory_references: true,
                supports_progress_reporting: true,
                supports_invalidated_event: true,
                supports_memory_event: true,
                supports_args_can_be_interpreted_by_shell: true,
            },
        })
        .await
        .expect("failed to send request");

    let re = client
        .responses
        .recv()
        .await
        .expect("a response was expected");

    let capabilities = match re.response {
        Response::Initialize { body } => Ok(body),
        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            "wrong response variant",
        )),
    }?;

    assert_eq!(ZkDap::capabilities(), capabilities);

    Ok(())
}

#[tokio::test]
async fn service_behavior() -> io::Result<()> {
    let path = std::env!("CARGO_MANIFEST_DIR");
    let path = std::path::PathBuf::from(path)
        .parent()
        .expect("failed to updir")
        .join("assets")
        .join("test.cdf")
        .display()
        .to_string();

    let (events, mut events_rx) = mpsc::channel(50);

    let service = ZkDap {
        events,
        backend: Arc::new(Mutex::new(None)),
    };

    service.initialize().await?;

    let request = ZkRequest::LoadCdf { path };
    let value = Value::from(request);
    let response = service.custom_request(Some(value)).await?;
    let response = ZkResponse::from(response);

    assert!(matches!(response, ZkResponse::LoadCdf));

    service.restart().await?;
    service.breakpoint_locations(None).await?;
    service.evaluate().await?;

    while events_rx.try_recv().is_ok() {}

    service.next().await?;
    service
        .goto(GotoArguments {
            thread_id: 0,
            target_id: 0,
        })
        .await?;
    service.r#continue().await?;
    service.reverse_continue().await?;
    service
        .add_breakpoint(Breakpoint {
            id: None,
            verified: true,
            message: None,
            source: Some(Source {
                name: Some("foo".into()),
                source_reference: Some(SourceReference::Path("bar".into())),
                presentation_hint: None,
                origin: None,
                sources: vec![],
                adapter_data: None,
                checksums: vec![],
            }),
            line: None,
            column: None,
            end_line: None,
            end_column: None,
            instruction_reference: None,
            offset: None,
        })
        .await?;
    service.remove_breakpoint(0).await?;
    service.source_contents().await?;
    service.scopes().await?;
    service
        .set_breakpoints(SetBreakpointsArguments {
            source: Source {
                name: Some("foo".into()),
                source_reference: Some(SourceReference::Path("bar".into())),
                presentation_hint: None,
                origin: None,
                sources: vec![],
                adapter_data: None,
                checksums: vec![],
            },
            breakpoints: vec![],
            lines: vec![],
            source_modified: true,
        })
        .await?;

    while events_rx.try_recv().is_ok() {}

    service.stack_trace().await?;
    service.step_back().await?;
    service.threads().await?;
    service
        .variables(VariablesArguments {
            variables_reference: 0,
            filter: None,
            start: None,
            count: None,
            format: None,
        })
        .await?;
    service.witness(0).await?;

    while events_rx.try_recv().is_ok() {}

    Ok(())
}
