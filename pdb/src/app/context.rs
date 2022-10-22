use std::collections::HashMap;
use std::sync::Arc;
use std::{fmt, io};

use dap_reactor::reactor::ClientRequest;
use dusk_cdf::ZkSource;
use tokio::sync::{mpsc, RwLock};

use crate::commands::Command;

use super::config::Config;
use super::Output;

#[derive(Clone)]
pub struct Context {
    config: Config,
    requests: mpsc::Sender<ClientRequest>,
    outputs: mpsc::Sender<Output>,
    contents_lock: mpsc::Sender<()>,
    inner: Arc<RwLock<ContextInner>>,
}

impl Context {
    pub fn new(
        config: Config,
        requests: mpsc::Sender<ClientRequest>,
        outputs: mpsc::Sender<Output>,
    ) -> Self {
        let (contents_lock, contents_lock_rx) = mpsc::channel(10);

        let inner = ContextInner::new(contents_lock_rx);
        let inner = Arc::new(RwLock::new(inner));

        Self {
            config,
            requests,
            outputs,
            contents_lock,
            inner,
        }
    }

    pub const fn config(&self) -> &Config {
        &self.config
    }

    pub async fn path(&self) -> Option<String> {
        self.inner.read().await.path.as_ref().cloned()
    }

    pub async fn receive_command(&self, command: Command) -> io::Result<()> {
        for request in command.into_iter() {
            self.send_request(request).await?;
        }

        Ok(())
    }

    pub async fn replace_path(
        &self,
        path: String,
    ) -> io::Result<Option<String>> {
        let previous = self.inner.write().await.path.replace(path.clone());

        self.receive_command(Command::Open { path }).await?;

        Ok(previous)
    }

    pub async fn contents(&self, name: &str) -> Option<String> {
        let mut inner = self.inner.write().await;

        if inner.locked {
            inner.contents_lock.recv().await;
            inner.locked = false;
        }

        inner.contents.get(name).cloned()
    }

    pub async fn replace_contents_batch<C>(&self, contents: C)
    where
        C: IntoIterator<Item = ZkSource>,
    {
        let contents = contents
            .into_iter()
            .map(|ZkSource { path, contents }| (path, contents));

        let mut inner = self.inner.write().await;

        inner.contents.clear();
        inner.contents.extend(contents);
    }

    pub async fn send_request<R>(&self, request: R) -> io::Result<()>
    where
        R: Into<ClientRequest>,
    {
        self.requests
            .send_timeout(request.into(), self.config.render_timeout())
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    pub async fn send_output<O>(&self, output: O) -> io::Result<()>
    where
        O: Into<Output>,
    {
        self.outputs
            .send_timeout(output.into(), self.config.render_timeout())
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    pub async fn send_error_output<E>(&self, error: E)
    where
        E: fmt::Display,
    {
        self.send_output(Output {
            contents: None,
            console: vec![],
            error: vec![error.to_string()],
        })
        .await
        .ok();
    }

    pub async fn lock_contents(&self) {
        self.inner.write().await.locked = true;
    }

    pub async fn unlock_contents(&self) {
        self.contents_lock.send(()).await.ok();
    }
}

#[derive(Debug)]
struct ContextInner {
    path: Option<String>,
    locked: bool,
    contents: HashMap<String, String>,
    contents_lock: mpsc::Receiver<()>,
}

impl ContextInner {
    pub fn new(contents_lock: mpsc::Receiver<()>) -> Self {
        Self {
            path: None,
            locked: false,
            contents: HashMap::new(),
            contents_lock,
        }
    }
}
