//! M033 lifecycle reconciliation and ownership tests.

#![cfg(feature = "i2pcontrol")]

use std::sync::Arc;

use emissary_cli::i2pcontrol::{
    control_plane::TunnelManagerControl,
    domain::tunnel::{
        StartIntent, TunnelDefinition, TunnelName, TunnelOptions, TunnelOwnership,
        TunnelRuntimeState, TunnelType,
    },
    production::{ProductionTunnelManagerControl, StartupClientConfig, StartupTunnelInventory},
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

fn client_definition(name: &str, start_intent: StartIntent) -> TunnelDefinition {
    TunnelDefinition {
        name: TunnelName::new(name).unwrap(),
        tunnel_type: TunnelType::Client,
        ownership: TunnelOwnership::ControlPlane,
        runtime_state: TunnelRuntimeState::Stopped,
        start_intent,
        options: TunnelOptions {
            target_destination: Some("destination".to_string()),
            listen_port: Some(0),
            ..Default::default()
        },
        raw_config: Default::default(),
    }
}

fn invalid_client_definition(name: &str) -> TunnelDefinition {
    let mut definition = client_definition(name, StartIntent::StartOnLoad);
    definition.options.target_destination = None;
    definition
}

async fn fake_sam() -> (u16, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let task = tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let (read_half, mut write_half) = stream.into_split();
                let mut reader = BufReader::new(read_half);
                let mut line = String::new();
                loop {
                    line.clear();
                    if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                        break;
                    }
                    let response = if line.starts_with("HELLO") {
                        "HELLO REPLY RESULT=OK VERSION=3.3\n"
                    } else if line.starts_with("SESSION CREATE") {
                        "SESSION STATUS DESTINATION=test-destination\n"
                    } else {
                        "STREAM STATUS RESULT=OK\n"
                    };
                    if write_half.write_all(response.as_bytes()).await.is_err() {
                        break;
                    }
                }
            });
        }
    });
    (port, task)
}

#[tokio::test]
async fn start_on_load_starts_eligible_client_and_isolates_failure() {
    let (sam_port, sam_task) = fake_sam().await;
    let directory = tempfile::tempdir().unwrap();
    let store_dir = directory.path().join("tunnels");

    let initial = ProductionTunnelManagerControl::new_with_startup_inventory_and_sam_port(
        store_dir.clone(),
        StartupTunnelInventory::default(),
        Some(sam_port),
    )
    .unwrap();
    initial.load().await.unwrap();
    initial.create(invalid_client_definition("bad-on-load")).await.unwrap();
    initial
        .create(client_definition("good-on-load", StartIntent::StartOnLoad))
        .await
        .unwrap();
    drop(initial);

    let manager = Arc::new(
        ProductionTunnelManagerControl::new_with_startup_inventory_and_sam_port(
            store_dir,
            StartupTunnelInventory::default(),
            Some(sam_port),
        )
        .unwrap(),
    );
    manager.load().await.unwrap();

    assert_eq!(
        manager.get("bad-on-load").await.unwrap().unwrap().runtime_state,
        TunnelRuntimeState::Stopped
    );
    assert_eq!(
        manager.get("good-on-load").await.unwrap().unwrap().runtime_state,
        TunnelRuntimeState::Running
    );

    let mut running = manager.get("good-on-load").await.unwrap().unwrap();
    running.options.description = Some("must-not-edit-while-running".to_string());
    assert!(manager.update("good-on-load", running, None).await.is_err());
    assert!(manager.delete("good-on-load").await.unwrap());
    assert!(manager.get("good-on-load").await.unwrap().is_none());

    sam_task.abort();
}

#[tokio::test]
async fn start_on_load_skips_unsupported_and_startup_managed_definitions() {
    let directory = tempfile::tempdir().unwrap();
    let store_dir = directory.path().join("tunnels");
    let startup = StartupTunnelInventory::from_configs(
        &[StartupClientConfig {
            name: "startup-client".to_string(),
            address: None,
            port: 1234,
            destination: "startup-destination".to_string(),
            destination_port: None,
        }],
        &[],
    )
    .unwrap();

    let initial = ProductionTunnelManagerControl::new_with_startup_inventory(
        store_dir.clone(),
        startup.clone(),
    )
    .unwrap();
    initial.load().await.unwrap();
    let mut unsupported = client_definition("unsupported-on-load", StartIntent::StartOnLoad);
    unsupported.tunnel_type = TunnelType::Socks;
    unsupported.ownership = TunnelOwnership::Unsupported;
    unsupported.runtime_state = TunnelRuntimeState::Unsupported;
    initial.create(unsupported).await.unwrap();
    drop(initial);

    let manager =
        ProductionTunnelManagerControl::new_with_startup_inventory(store_dir, startup).unwrap();
    manager.load().await.unwrap();

    assert_eq!(
        manager.get("startup-client").await.unwrap().unwrap().runtime_state,
        TunnelRuntimeState::ExternallyManaged
    );
    assert_eq!(
        manager.get("unsupported-on-load").await.unwrap().unwrap().runtime_state,
        TunnelRuntimeState::Unsupported
    );
}

#[tokio::test]
async fn restart_reloads_latest_stopped_definition_and_delete_stops_first() {
    let (sam_port, sam_task) = fake_sam().await;
    let directory = tempfile::tempdir().unwrap();
    let manager = ProductionTunnelManagerControl::new_with_startup_inventory_and_sam_port(
        directory.path().join("tunnels"),
        StartupTunnelInventory::default(),
        Some(sam_port),
    )
    .unwrap();
    manager.load().await.unwrap();
    manager
        .create(client_definition("restartable", StartIntent::DoNotStart))
        .await
        .unwrap();
    assert_eq!(manager.start("restartable").await.unwrap(), "ok");
    assert_eq!(manager.stop("restartable").await.unwrap(), "ok");

    let mut edited = manager.get("restartable").await.unwrap().unwrap();
    edited.options.target_destination = Some("new-destination".to_string());
    manager.update("restartable", edited, None).await.unwrap();
    assert_eq!(manager.restart("restartable").await.unwrap(), "ok");
    assert!(manager.delete("restartable").await.unwrap());
    assert!(manager.get("restartable").await.unwrap().is_none());

    sam_task.abort();
}
