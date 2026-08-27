//! Common Proposal 170 session option translation.
//!
//! This module is the single I2PControl-owned boundary between canonical
//! tunnel options and Yosemite's `SessionOptions`. It intentionally exposes
//! no Proposal 170 types to the router or core crates.

use yosemite::{DestinationKind, SessionOptions};

use super::super::{options::validate_common_options, BackendError, BackendResult};
use crate::i2pcontrol::domain::tunnel::{TunnelDefinition, TunnelType};

/// Build the Yosemite session settings for one validated definition.
///
/// The returned settings are safe to move into a runtime task. Persistent
/// private key material is carried only by Yosemite's redacted
/// `DestinationKind` and never by a public domain response.
pub fn build_session_options(
    definition: &TunnelDefinition,
    sam_tcp_port: u16,
    publish: bool,
    destination: DestinationKind,
) -> BackendResult<SessionOptions> {
    validate_common_options(definition.tunnel_type, &definition.options).map_err(option_error)?;

    if definition.options.tunnel_length.is_some_and(|value| value > 3) {
        return Err(BackendError::UnsupportedOption {
            tunnel_type: definition.tunnel_type,
            option: "TunnelLength".to_owned(),
        });
    }
    if definition
        .options
        .tunnel_quantity
        .is_some_and(|value| !(1..=6).contains(&value))
    {
        return Err(BackendError::UnsupportedOption {
            tunnel_type: definition.tunnel_type,
            option: "TunnelQuantity".to_owned(),
        });
    }

    let mut options = SessionOptions {
        nickname: definition.name.as_str().to_owned(),
        samv3_tcp_port: sam_tcp_port,
        publish,
        destination,
        ..Default::default()
    };

    options.inbound_len = definition.options.tunnel_length.unwrap_or(3) as usize;
    options.outbound_len = options.inbound_len;
    options.inbound_quantity = definition.options.tunnel_quantity.unwrap_or(2) as usize;
    options.outbound_quantity = options.inbound_quantity;

    let enc_type = definition
        .options
        .enc_type
        .clone()
        .or_else(|| definition.options.i2cp_options.get("leaseSetEncType").cloned());
    if let Some(value) = definition.options.enc_type.as_deref() {
        validate_encryption_type(definition.tunnel_type, value)?;
    }
    options.lease_set_enc_type = enc_type;

    Ok(options)
}

fn validate_encryption_type(tunnel_type: TunnelType, value: &str) -> BackendResult<()> {
    let mut types = Vec::new();
    for item in value.split(',') {
        let parsed = item.parse::<u16>().map_err(|_| BackendError::Internal {
            message: "EncType is invalid".to_owned(),
        })?;
        if !(3..=7).contains(&parsed) || types.contains(&parsed) {
            return Err(BackendError::UnsupportedOption {
                tunnel_type,
                option: "EncType".to_owned(),
            });
        }
        types.push(parsed);
    }
    if types.is_empty() || types.len() > 2 || (types.len() == 2 && !types.contains(&4)) {
        return Err(BackendError::UnsupportedOption {
            tunnel_type,
            option: "EncType".to_owned(),
        });
    }
    Ok(())
}

fn option_error(error: super::super::options::OptionValidationError) -> BackendError {
    match error {
        super::super::options::OptionValidationError::Missing {
            tunnel_type,
            option,
        } => BackendError::MissingOption {
            tunnel_type,
            option: option.to_owned(),
        },
        super::super::options::OptionValidationError::Unsupported {
            tunnel_type,
            option,
        } => BackendError::UnsupportedOption {
            tunnel_type,
            option,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i2pcontrol::domain::tunnel::{
        StartIntent, TunnelName, TunnelOptions, TunnelOwnership, TunnelRuntimeState,
    };
    use std::collections::BTreeMap;

    fn definition() -> TunnelDefinition {
        TunnelDefinition {
            name: TunnelName::new("common-session").unwrap(),
            tunnel_type: TunnelType::Client,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: StartIntent::DoNotStart,
            options: TunnelOptions::default(),
            raw_config: BTreeMap::new(),
        }
    }

    #[test]
    fn typed_session_options_translate_deterministically() {
        let mut definition = definition();
        definition.options.tunnel_length = Some(2);
        definition.options.tunnel_quantity = Some(4);
        definition.options.enc_type = Some("4,3".to_owned());
        let options =
            build_session_options(&definition, 7656, false, DestinationKind::Transient).unwrap();
        assert_eq!(options.inbound_len, 2);
        assert_eq!(options.outbound_len, 2);
        assert_eq!(options.inbound_len_variance, 0);
        assert_eq!(options.outbound_backup_quantity, 0);
        assert_eq!(options.lease_set_enc_type.as_deref(), Some("4,3"));
    }

    #[test]
    fn unsupported_signing_type_fails_without_downgrade() {
        let mut definition = definition();
        definition.options.sig_type = Some("1".to_owned());
        let error = build_session_options(&definition, 7656, false, DestinationKind::Transient)
            .unwrap_err();
        assert!(
            matches!(error, BackendError::UnsupportedOption { option, .. } if option == "SigType")
        );
    }

    #[test]
    fn unsupported_session_controls_fail_before_translation() {
        for set in [
            |options: &mut TunnelOptions| options.tunnel_variance = Some(1),
            |options: &mut TunnelOptions| options.tunnel_backup_quantity = Some(1),
            |options: &mut TunnelOptions| options.use_ssl = Some(true),
            |options: &mut TunnelOptions| options.shared = Some(true),
            |options: &mut TunnelOptions| options.new_dest = Some(true),
            |options: &mut TunnelOptions| options.persistent_client_key = Some(true),
        ] {
            let mut definition = definition();
            set(&mut definition.options);
            assert!(
                build_session_options(&definition, 7656, false, DestinationKind::Transient)
                    .is_err()
            );
        }
    }
}
