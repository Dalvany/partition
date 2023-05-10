use crate::common::{PartitionWorld, CONFIGURATION_FILE};
use cucumber::{then, World};
use futures::FutureExt as _;
use hyper::StatusCode;
use std::future;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

mod common;

#[then(expr = "the HTTP status is {int}")]
async fn check_status(world: &mut PartitionWorld, expected_status: u16) {
    assert_eq!(world.status(), StatusCode::from_u16(expected_status).ok())
}

#[then(expr = "header {string} is {string}")]
async fn check_header(world: &mut PartitionWorld, header: String, expected_location: String) {
    let location = world.header(&header);
    assert_eq!(location, expected_location);
}

#[tokio::main]
async fn main() {
    PartitionWorld::cucumber()
        .after(|_feature, _rule, _scenario, _ev, world| {
            if let Some(world) = world {
                world.stop();
            }

            future::ready(()).boxed()
        })
        .before(|_feature, _rule, _scenario, world| {
            let path = std::env::current_dir().unwrap().join(CONFIGURATION_FILE);
            let path_str = path.to_str().unwrap();
            let _ = std::fs::remove_dir_all("target/partition");
            std::fs::create_dir_all("target/partition")
                .expect("Can't create test temporary directory");
            let result = Command::new("target/debug/partition-server")
                .args(["-c", path_str])
                .spawn()
                .expect("Can't run partition server");
            world.process(result);

            // Wait that server is actually up.
            sleep(Duration::from_secs(3)).boxed_local()
        })
        .run("tests/features/ui")
        .await;
}
