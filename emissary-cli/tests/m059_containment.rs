//! M059 original CLI/runtime containment guards.

#![cfg(feature = "i2pcontrol")]

use std::path::Path;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root")
}

fn production_source(path: &str) -> String {
    let source = std::fs::read_to_string(workspace_root().join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
    source.split("#[cfg(test)]").next().unwrap_or(&source).to_string()
}

#[test]
fn legacy_address_book_has_only_a_neutral_runtime_hook() {
    let source = production_source("emissary-cli/src/address_book.rs");
    for forbidden in [
        "JsonRpc",
        "RuntimeAddressBookSnapshot",
        "RuntimeAddressBookEntry",
        "RuntimeAddressBookType",
        "control-state.json",
        "Proposal 170",
        "I2PControl",
    ] {
        assert!(!source.contains(forbidden), "legacy AddressBook owns {forbidden}");
    }
    assert!(source.contains("AddressBookRuntimeHook"));
}

#[test]
fn proxy_and_tunnel_runtime_modules_do_not_own_control_plane_types() {
    for path in [
        "emissary-cli/src/proxy/http/mod.rs",
        "emissary-cli/src/proxy/http/error.rs",
        "emissary-cli/src/proxy/http/request.rs",
        "emissary-cli/src/proxy/socks.rs",
        "emissary-cli/src/tunnel/client.rs",
        "emissary-cli/src/tunnel/server.rs",
    ] {
        let source = production_source(path);
        for forbidden in [
            "JsonRpc",
            "ClientServicesInfo",
            "TunnelManagerControl",
            "control-state",
        ] {
            assert!(!source.contains(forbidden), "{path} owns {forbidden}");
        }
    }
}

#[test]
fn core_is_outside_the_m059_changed_path_budget() {
    let output = std::process::Command::new("git")
        .args([
            "diff",
            "--name-only",
            "adb2f52543764b267b2bcb282d093111001ae4b2",
            "--",
            "emissary-core",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("git diff");
    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "M059 changed an emissary-core path"
    );
}
