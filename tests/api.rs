use crate::common::{PartitionWorld, CONFIGURATION_FILE};
use cucumber::{then, World};
use futures::FutureExt;
use reqwest::StatusCode;
use server_lib::models::Informations;
use std::process::Command;
use std::time::Duration;
use std::{env, future};
use tokio::time::sleep;

mod common;

#[then("version match Cargo.toml")]
async fn check_api_root(world: &mut PartitionWorld) {
    assert_eq!(world.status(), StatusCode::from_u16(200).ok(),);

    let result = world.content::<Informations>().await;

    assert!(
        result.is_ok(),
        "Deserialization returned an error : {:?}.",
        result.unwrap_err(),
    );

    let expected = Informations {
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
    };
    assert_eq!(
        result.unwrap(),
        expected,
        "Deserialization returned an error.",
    );
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
            let path = env::current_dir().unwrap().join(CONFIGURATION_FILE);
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
        .run("tests/features/api")
        .await;
}
