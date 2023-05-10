use cucumber::{given, when, World};
use hyper::StatusCode;
use reqwest::redirect::Policy;
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use std::process::{Child, ExitStatus};
use std::sync::Arc;

pub static CONFIGURATION_FILE: &str = "tests-resources/config.toml";

#[derive(Debug, Default, World)]
pub struct PartitionWorld {
    client: Option<Client>,
    process: Option<Arc<Child>>,
    response: Option<Response>,
}

impl PartitionWorld {
    pub fn stop(&mut self) {
        if let Some(child) = self.process.as_mut() {
            Arc::<Child>::get_mut(child)
                .expect("Can't get child process")
                .kill()
                .expect("Can't stop partition server");
        }
    }

    pub fn process(&mut self, process: Child) {
        self.process = Some(Arc::new(process))
    }

    pub fn response(&mut self, response: Response) {
        self.response = Some(response)
    }

    pub fn status(&self) -> Option<StatusCode> {
        self.response.as_ref().map(|v| v.status())
    }

    pub fn header(&self, header: &str) -> String {
        self.response
            .as_ref()
            .and_then(|v| v.headers().get(header))
            .map(|v| v.to_str().unwrap().to_string())
            .unwrap_or_default()
    }

    #[allow(dead_code)]
    pub async fn content<T: DeserializeOwned>(&mut self) -> reqwest::Result<T> {
        let reponse = self.response.take().expect("Can't get body");
        reponse.json::<T>().await
    }

    pub fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        if let Some(child) = self.process.as_mut() {
            Arc::<Child>::get_mut(child)
                .expect("Can't get child process")
                .try_wait()
        } else {
            Ok(None)
        }
    }
}

#[given(expr = "partition is running")]
pub async fn is_running(world: &mut PartitionWorld) {
    assert!(world.process.is_some(), "Process isn't running");
    let running = world.try_wait();
    if let Ok(Some(status)) = running {
        panic!("Process exited with status {status}.");
    } else if let Err(error) = running {
        panic!("Error {error:?}");
    }
}

#[when(expr = "accessing {string}")]
async fn access_url(world: &mut PartitionWorld, path: String) {
    if world.client.is_none() {
        world.client = Some(Client::builder().redirect(Policy::none()).build().unwrap());
    }

    let url = format!("http://127.0.0.1:8000{path}");
    match world.client.as_ref().unwrap().get(&url).send().await {
        Ok(response) => world.response(response),
        Err(error) => panic!("Error accessing url '{url}' : {error:?}"),
    }
}
