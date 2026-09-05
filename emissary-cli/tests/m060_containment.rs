//! M060 core observation containment guards.

#![cfg(feature = "i2pcontrol")]

use std::{collections::BTreeSet, path::Path};

const UPSTREAM_BASELINE: &str = "9b43484a21d5a1291c4881cdae62a36c527f8c0f";
const M060_IMPLEMENTATION_HEAD: &str = "6085eca";

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root")
}

fn source(path: &str) -> String {
    std::fs::read_to_string(workspace_root().join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn core_changes_stay_inside_the_accepted_m060_budget() {
    let allowed = BTreeSet::from([
        "emissary-core/src/config.rs",
        "emissary-core/src/error/mod.rs",
        "emissary-core/src/events.rs",
        "emissary-core/src/i2cp/socket.rs",
        "emissary-core/src/inspection.rs",
        "emissary-core/src/lib.rs",
        "emissary-core/src/primitives/router_identity.rs",
        "emissary-core/src/router/context.rs",
        "emissary-core/src/router/mod.rs",
        "emissary-core/src/runtime/mod.rs",
        "emissary-core/src/sam/mod.rs",
        "emissary-core/src/sam/parser.rs",
        "emissary-core/src/sam/pending/connection.rs",
        "emissary-core/src/sam/protocol/streaming/listener.rs",
        "emissary-core/src/sam/protocol/streaming/mod.rs",
        "emissary-core/src/sam/session.rs",
        "emissary-core/src/sam/socket.rs",
        "emissary-core/src/subsystem/mod.rs",
        "emissary-core/src/transport/mod.rs",
        "emissary-core/src/transport/ntcp2/mod.rs",
        "emissary-core/src/transport/ntcp2/session/active.rs",
        "emissary-core/src/transport/ntcp2/session/mod.rs",
        "emissary-core/src/transport/ssu2/message/data.rs",
        "emissary-core/src/transport/ssu2/mod.rs",
        "emissary-core/src/transport/ssu2/peer_test/mod.rs",
        "emissary-core/src/transport/ssu2/relay/mod.rs",
        "emissary-core/src/transport/ssu2/session/active/mod.rs",
        "emissary-core/src/transport/ssu2/session/pending/inbound.rs",
        "emissary-core/src/transport/ssu2/session/terminating.rs",
        "emissary-core/src/transport/ssu2/socket.rs",
        "emissary-core/src/tunnel/mod.rs",
        "emissary-core/src/tunnel/pool/mod.rs",
        "emissary-core/src/tunnel/transit/mod.rs",
    ])
    .into_iter()
    .map(String::from)
    .collect::<BTreeSet<_>>();
    let output = std::process::Command::new("git")
        .args([
            "diff",
            "--name-only",
            UPSTREAM_BASELINE,
            M060_IMPLEMENTATION_HEAD,
            "--",
            "emissary-core",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("git diff");
    assert!(output.status.success());
    let changed = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    assert!(changed.is_subset(&allowed), "M060 changed an unbudgeted core path");
}

#[test]
fn core_observation_surface_is_neutral_and_no_longer_aggregated() {
    for path in [
        "emissary-core/src/events.rs",
        "emissary-core/src/inspection.rs",
        "emissary-core/src/primitives/router_identity.rs",
        "emissary-core/src/router/context.rs",
        "emissary-core/src/router/mod.rs",
        "emissary-core/src/sam/mod.rs",
    ] {
        let contents = source(path);
        let text = contents.split("#[cfg(test)]").next().unwrap_or_default();
        for forbidden in ["I2PControl", "Proposal 170", "JsonRpc", "ClientServicesInfo"] {
            assert!(!text.contains(forbidden), "{path} contains control-plane term {forbidden}");
        }
    }
    let inspection = source("emissary-core/src/inspection.rs");
    assert!(!inspection.contains("pub struct CoreSnapshot"));
    assert!(!inspection.contains("pub struct TransportSnapshot"));
    assert!(!inspection.contains("pub struct TunnelSnapshot"));
    assert!(!inspection.contains("pub struct NetDbSnapshot"));
}

#[test]
fn observation_failure_and_hot_path_accounting_remain_passive() {
    let sam = source("emissary-core/src/sam/session.rs");
    assert!(sam.contains("observation_hook"));
    assert!(sam.contains("observation hook rejected socket activation"));
    assert!(!sam.contains("return Err(error)"));

    let ssu2 = source("emissary-core/src/transport/ssu2/session/active/mod.rs");
    let send_block = ssu2
        .split("while let Some((pkt, destination)) = self.write_buffer.pop_front()")
        .nth(1)
        .and_then(|text| text.split("// only drain more packets").next())
        .expect("SSU2 buffered send block");
    assert_eq!(
        send_block.matches("record_peer_bytes").count(),
        1,
        "buffered SSU2 sends must be counted once"
    );
}
