//! M023 startup inventory, ownership, and address provenance tests.

#![cfg(feature = "i2pcontrol")]

use std::sync::Arc;

use emissary_cli::i2pcontrol::{
    client_services::assemble_response,
    control_plane::TunnelManagerControl,
    domain::tunnel::{
        StartIntent, TunnelDefinition, TunnelName, TunnelOptions, TunnelOwnership,
        TunnelRuntimeState, TunnelType,
    },
    production::{
        ProductionTunnelManagerControl, StartupClientConfig, StartupServerConfig,
        StartupTunnelInventory,
    },
    service_registry::ServiceRegistry,
};

fn startup_inventory() -> StartupTunnelInventory {
    StartupTunnelInventory::from_configs(
        &[StartupClientConfig {
            name: "startup-client".into(),
            address: Some("127.0.0.1".into()),
            port: 4444,
            destination: "client-destination".into(),
            destination_port: Some(80),
        }],
        &[StartupServerConfig {
            name: "startup-server".into(),
            port: 8080,
        }],
    )
    .unwrap()
}

fn control_definition(name: &str) -> TunnelDefinition {
    TunnelDefinition {
        name: TunnelName::new(name).unwrap(),
        tunnel_type: TunnelType::Client,
        ownership: TunnelOwnership::ControlPlane,
        runtime_state: TunnelRuntimeState::Stopped,
        start_intent: StartIntent::DoNotStart,
        options: TunnelOptions {
            target_destination: Some("control-destination".into()),
            ..Default::default()
        },
        raw_config: Default::default(),
    }
}

#[test]
fn startup_mapping_is_bounded_deterministic_and_read_only() {
    let inventory = startup_inventory();
    let definitions = inventory.list().unwrap();
    assert_eq!(
        definitions
            .iter()
            .map(|definition| definition.name.as_str())
            .collect::<Vec<_>>(),
        vec!["startup-client", "startup-server"]
    );
    assert!(definitions
        .iter()
        .all(|definition| definition.ownership == TunnelOwnership::StartupManaged));
    assert!(definitions
        .iter()
        .all(|definition| definition.runtime_state == TunnelRuntimeState::ExternallyManaged));
    assert_eq!(
        definitions[0].options.target_destination.as_deref(),
        Some("client-destination")
    );
    assert_eq!(
        definitions[0].options.listen_interface.as_deref(),
        Some("127.0.0.1")
    );
    assert!(definitions[1].options.hosting_destination.is_none());
}

#[test]
fn startup_mapping_rejects_cross_type_name_collisions_and_oversize() {
    let collision = StartupTunnelInventory::from_configs(
        &[StartupClientConfig {
            name: "same".into(),
            address: None,
            port: 1,
            destination: "destination".into(),
            destination_port: None,
        }],
        &[StartupServerConfig {
            name: "same".into(),
            port: 2,
        }],
    );
    assert!(collision.is_err());

    let clients = (0..1001)
        .map(|index| StartupClientConfig {
            name: format!("client-{index}"),
            address: None,
            port: index as u16,
            destination: "destination".into(),
            destination_port: None,
        })
        .collect::<Vec<_>>();
    assert!(StartupTunnelInventory::from_configs(&clients, &[]).is_err());
}

#[tokio::test]
async fn production_inventory_combines_restartably_and_rejects_startup_mutations() {
    let directory = tempfile::tempdir().unwrap();
    let inventory = startup_inventory();
    inventory
        .publish_server_destination("startup-server", "actual-server-destination")
        .unwrap();
    let manager = ProductionTunnelManagerControl::new_with_startup_inventory(
        directory.path().join("tunnels"),
        inventory.clone(),
    )
    .unwrap();
    manager.load().await.unwrap();
    manager.create(control_definition("control")).await.unwrap();

    let definitions = manager.list().await.unwrap();
    assert_eq!(definitions.len(), 3);
    assert_eq!(definitions[0].name.as_str(), "control");
    assert_eq!(definitions[1].name.as_str(), "startup-client");
    assert_eq!(definitions[2].name.as_str(), "startup-server");
    assert_eq!(
        manager
            .get("startup-server")
            .await
            .unwrap()
            .unwrap()
            .options
            .hosting_destination
            .as_deref(),
        Some("actual-server-destination")
    );
    assert!(manager.create(control_definition("startup-client")).await.is_err());
    assert!(manager
        .update(
            "control",
            control_definition("control"),
            Some(TunnelName::new("startup-server").unwrap()),
        )
        .await
        .is_err());
    assert!(manager.delete("startup-client").await.is_err());
    assert!(manager.start("startup-client").await.is_err());

    let restarted = ProductionTunnelManagerControl::new_with_startup_inventory(
        directory.path().join("tunnels"),
        inventory,
    )
    .unwrap();
    restarted.load().await.unwrap();
    assert_eq!(restarted.list().await.unwrap().len(), 3);
}

#[tokio::test]
async fn persisted_startup_name_collision_fails_closed() {
    let directory = tempfile::tempdir().unwrap();
    let persisted = ProductionTunnelManagerControl::new(directory.path().join("tunnels")).unwrap();
    persisted.load().await.unwrap();
    persisted.create(control_definition("startup-client")).await.unwrap();

    let collision = ProductionTunnelManagerControl::new_with_startup_inventory(
        directory.path().join("tunnels"),
        startup_inventory(),
    )
    .unwrap();
    let error = collision.load().await.unwrap_err();
    assert!(error.contains("colliding name"));
}

#[tokio::test]
async fn client_services_uses_actual_destinations_and_errors_when_missing() {
    let inventory = startup_inventory();
    let directory = tempfile::tempdir().unwrap();
    let manager = Arc::new(
        ProductionTunnelManagerControl::new_with_startup_inventory(
            directory.path().join("tunnels"),
            inventory.clone(),
        )
        .unwrap(),
    );
    manager.load().await.unwrap();
    let error = assemble_response(
        &ServiceRegistry::new().snapshot(),
        &["I2PTunnel"],
        manager.as_ref(),
    )
    .await
    .unwrap_err();
    assert!(error.contains("no actual I2P destination"));

    inventory
        .publish_server_destination("startup-server", "actual-server-destination")
        .unwrap();
    let response = assemble_response(
        &ServiceRegistry::new().snapshot(),
        &["I2PTunnel"],
        manager.as_ref(),
    )
    .await
    .unwrap();
    assert_eq!(
        response["I2PTunnel"]["client"]["startup-client"]["address"],
        "client-destination"
    );
    assert_eq!(
        response["I2PTunnel"]["server"]["startup-server"]["address"],
        "actual-server-destination"
    );
    assert_eq!(
        response["I2PTunnel"]["server"]["startup-server"]["port"],
        8080
    );
    assert_ne!(
        response["I2PTunnel"]["server"]["startup-server"]["address"],
        "127.0.0.1"
    );
}
