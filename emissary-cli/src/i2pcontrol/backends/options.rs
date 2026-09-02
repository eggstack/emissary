//! Backend-local capability validation for persisted Proposal 170 options.
//!
//! This is intentionally a small declaration/validator pair rather than a
//! global feature matrix. A backend must explicitly classify every
//! runtime-relevant option it accepts; management metadata remains outside
//! the runtime capability boundary.

use std::fmt;

use crate::i2pcontrol::domain::tunnel::{TunnelOptions, TunnelType};
use yosemite_i2pcontrol::SessionOption;

const SUPPORTED_SIGNATURE_TYPE: u16 = 7;
const MAX_CUSTOM_OPTIONS: usize = 32;
const MAX_CUSTOM_OPTION_KEY_LENGTH: usize = 64;
const MAX_CUSTOM_OPTION_VALUE_LENGTH: usize = 256;

/// Policy for the protocol-defined custom option namespace.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomOptionPolicy {
    /// Accept and pass the namespace to the backend's protocol adapter.
    Accept,
    /// Reject any requested custom option until the backend implements it.
    Reject,
}

/// A bounded backend capability declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionCapabilities {
    pub required: &'static [&'static str],
    pub required_any: &'static [&'static str],
    pub optional: &'static [&'static str],
    pub i2cp: CustomOptionPolicy,
    pub custom: CustomOptionPolicy,
}

impl OptionCapabilities {
    pub const fn new(
        required: &'static [&'static str],
        required_any: &'static [&'static str],
        optional: &'static [&'static str],
        i2cp: CustomOptionPolicy,
        custom: CustomOptionPolicy,
    ) -> Self {
        Self {
            required,
            required_any,
            optional,
            i2cp,
            custom,
        }
    }
}

/// Deterministic, sanitized validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionValidationError {
    Missing {
        tunnel_type: TunnelType,
        option: &'static str,
    },
    Unsupported {
        tunnel_type: TunnelType,
        option: String,
    },
}

impl fmt::Display for OptionValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing {
                tunnel_type,
                option,
            } => {
                write!(f, "{} requires option {option}", tunnel_type.as_str())
            }
            Self::Unsupported {
                tunnel_type,
                option,
            } => {
                write!(
                    f,
                    "{} does not support option {option}",
                    tunnel_type.as_str()
                )
            }
        }
    }
}

impl std::error::Error for OptionValidationError {}

/// Validate a definition's runtime-relevant options before resource allocation.
pub fn validate_options(
    tunnel_type: TunnelType,
    options: &TunnelOptions,
    capabilities: OptionCapabilities,
) -> Result<(), OptionValidationError> {
    for &field in capabilities.required {
        if !field_present(options, field) {
            return Err(OptionValidationError::Missing {
                tunnel_type,
                option: field,
            });
        }
    }

    if !capabilities.required_any.is_empty()
        && !capabilities.required_any.iter().any(|field| field_present(options, field))
    {
        return Err(OptionValidationError::Missing {
            tunnel_type,
            option: capabilities.required_any[0],
        });
    }

    for field in present_runtime_fields(options) {
        if is_common_runtime_field(field) {
            continue;
        }
        if !capabilities.optional.contains(&field)
            && !capabilities.required.contains(&field)
            && !capabilities.required_any.contains(&field)
        {
            return Err(OptionValidationError::Unsupported {
                tunnel_type,
                option: field.to_owned(),
            });
        }
    }

    if !options.i2cp_options.is_empty() && capabilities.i2cp == CustomOptionPolicy::Reject {
        return Err(OptionValidationError::Unsupported {
            tunnel_type,
            option: "I2CPOptions".to_owned(),
        });
    }
    if !options.custom_options.is_empty() && capabilities.custom == CustomOptionPolicy::Reject {
        return Err(OptionValidationError::Unsupported {
            tunnel_type,
            option: "CustomOptions".to_owned(),
        });
    }

    Ok(())
}

/// Validate the common Proposal 170 session/key fields shared by the real
/// backends. This check is deliberately separate from each protocol backend's
/// capability declaration so every backend uses the same ranges and mapping.
pub fn validate_common_options(
    tunnel_type: TunnelType,
    options: &TunnelOptions,
) -> Result<(), OptionValidationError> {
    let is_streamr = matches!(
        tunnel_type,
        TunnelType::StreamrClient | TunnelType::StreamrServer
    );

    // Proposal UseSSL controls the local application/session presentation side.
    // Yosemite's similarly named field controls TLS on the SAM control
    // connection, so it is deliberately not mapped here.
    if options.use_ssl.is_some() {
        return Err(common_unsupported(tunnel_type, "UseSSL"));
    }
    for (present, field) in [
        (
            options.priv_key_file.is_some()
                && !matches!(
                    tunnel_type,
                    TunnelType::Client
                        | TunnelType::HttpClient
                        | TunnelType::IrcClient
                        | TunnelType::Socks
                        | TunnelType::SocksIrc
                        | TunnelType::ConnectClient
                        | TunnelType::Server
                        | TunnelType::HttpServer
                        | TunnelType::HttpBidirServer
                        | TunnelType::IrcServer
                ),
            "PrivKeyFile",
        ),
    ] {
        if present {
            return Err(common_unsupported(tunnel_type, field));
        }
    }

    if let Some(value) = options.tunnel_variance {
        if is_streamr {
            return Err(common_unsupported(tunnel_type, "TunnelVariance"));
        }
        if !(-2..=2).contains(&value) {
            return Err(common_unsupported(tunnel_type, "TunnelVariance"));
        }
    }
    if let Some(value) = options.tunnel_backup_quantity {
        if is_streamr {
            return Err(common_unsupported(tunnel_type, "TunnelBackupQuantity"));
        }
        if value > 3 {
            return Err(common_unsupported(tunnel_type, "TunnelBackupQuantity"));
        }
    }
    if let Some(value) = options.sig_type.as_deref() {
        if is_streamr {
            return Err(common_unsupported(tunnel_type, "SigType"));
        }
        if value.parse::<u16>().ok() != Some(SUPPORTED_SIGNATURE_TYPE) || value != "7" {
            return Err(common_unsupported(tunnel_type, "SigType"));
        }
    }
    if !options.custom_options.is_empty() && !valid_custom_options(&options.custom_options) {
        return Err(common_unsupported(tunnel_type, "CustomOptions"));
    }
    if is_streamr && !options.custom_options.is_empty() {
        return Err(common_unsupported(tunnel_type, "CustomOptions"));
    }

    for (present, field) in [
        (options.tunnel_length.is_some(), "TunnelLength"),
        (options.tunnel_quantity.is_some(), "TunnelQuantity"),
        (options.enc_type.is_some(), "EncType"),
    ] {
        if present && is_streamr {
            return Err(common_unsupported(tunnel_type, field));
        }
    }
    // NewDest is coupled to M112's close-on-idle/resume lifecycle owner. It is
    // only meaningful for streaming client generations; in particular, do not
    // rotate a destination merely because a manual start was staged.
    if options.new_dest.is_some() && (!tunnel_type.is_client() || is_streamr) {
        return Err(common_unsupported(tunnel_type, "NewDest"));
    }
    // PersistentClientKey and Shared are meaningful only for the control-plane
    // client families and are applied by the bounded owner.
    if options.shared.is_some() && !tunnel_type.is_client() {
        return Err(common_unsupported(tunnel_type, "Shared"));
    }
    if options.persistent_client_key.is_some() && !tunnel_type.is_client() {
        return Err(common_unsupported(tunnel_type, "PersistentClientKey"));
    }

    Ok(())
}

fn common_unsupported(tunnel_type: TunnelType, option: &'static str) -> OptionValidationError {
    OptionValidationError::Unsupported {
        tunnel_type,
        option: option.to_owned(),
    }
}

fn is_common_runtime_field(field: &str) -> bool {
    matches!(
        field,
        "Shared"
            | "UseSSL"
            | "TunnelLength"
            | "TunnelVariance"
            | "TunnelQuantity"
            | "TunnelBackupQuantity"
            | "SigType"
            | "EncType"
            | "NewDest"
            | "PersistentClientKey"
            | "PrivKeyFile"
            | "CustomOptions"
    )
}

/// Runtime fields supported by the existing generic client backend.
pub const CLIENT_OPTIONS: OptionCapabilities = OptionCapabilities::new(
    &["TargetDestination", "ListenPort"],
    &[],
    &["TargetPort", "ListenInterface", "DelayOpen"],
    CustomOptionPolicy::Reject,
    CustomOptionPolicy::Accept,
);

/// Runtime fields supported by the existing generic server backend.
///
/// `i2cp` is `Accept` so the backend-specific `ServerTunnelBackend::validate_i2cp_options`
/// can be the authoritative allowlist (currently `leaseSetEncType` only). The
/// generic `validate_options` check stays coarse-grained; the runtime decides
/// which I2CP keys it can actually apply.
pub const SERVER_OPTIONS: OptionCapabilities = OptionCapabilities::new(
    &[],
    &["TargetPort", "ListenPort"],
    &["TargetHost", "Host"],
    CustomOptionPolicy::Accept,
    CustomOptionPolicy::Accept,
);

/// Proposal 170 fields consumed by the filtered IRC client runtime.
///
/// IRC automation fields are deliberately rejected: this backend forwards an
/// explicitly configured I2P destination and does not synthesize registration
/// or channel commands on behalf of a user.
pub const IRC_CLIENT_OPTIONS: OptionCapabilities = OptionCapabilities::new(
    &["TargetDestination", "ListenPort"],
    &[],
    &["TargetPort", "ListenInterface", "DelayOpen"],
    CustomOptionPolicy::Reject,
    CustomOptionPolicy::Accept,
);

/// Proposal 170 fields consumed by the filtered IRC server runtime.
pub const IRC_SERVER_OPTIONS: OptionCapabilities = OptionCapabilities::new(
    &[],
    &["TargetPort", "ListenPort"],
    &["HostingDestination"],
    CustomOptionPolicy::Reject,
    CustomOptionPolicy::Accept,
);

/// Proposal 170 HTTP proxy options consumed by the control-plane client.
pub const HTTP_CLIENT_OPTIONS: OptionCapabilities = OptionCapabilities::new(
    &["ListenPort"],
    &[],
    &[
        "ListenInterface",
        "ProxyUsername",
        "ProxyPassword",
        "DelayOpen",
    ],
    CustomOptionPolicy::Reject,
    CustomOptionPolicy::Accept,
);

/// The composed HTTP bidirectional server accepts the union of the already
/// implemented server and local-client typed fields. Outproxy fields are not
/// part of this capability declaration and are rejected by its backend before
/// any session or listener allocation.
pub const HTTP_BIDIR_SERVER_OPTIONS: OptionCapabilities = OptionCapabilities::new(
    &["TargetPort", "ListenPort"],
    &[],
    &[
        "ListenInterface",
        "HostingDestination",
        "AccessList",
        "HttpHost",
        "ProxyUsername",
        "ProxyPassword",
    ],
    CustomOptionPolicy::Reject,
    CustomOptionPolicy::Accept,
);

/// Proposal 170 CONNECT proxy options consumed by the control-plane client.
pub const CONNECT_CLIENT_OPTIONS: OptionCapabilities = OptionCapabilities::new(
    &["ListenPort"],
    &[],
    &[
        "ListenInterface",
        "ProxyUsername",
        "ProxyPassword",
        "DelayOpen",
    ],
    CustomOptionPolicy::Reject,
    CustomOptionPolicy::Accept,
);

/// Proposal 170 SOCKS frontend options. The target is selected by each
/// SOCKS request, so a persisted TargetDestination is deliberately not part
/// of this capability set.
pub const SOCKS_OPTIONS: OptionCapabilities = OptionCapabilities::new(
    &["ListenPort"],
    &[],
    &[
        "ListenInterface",
        "ProxyUsername",
        "ProxyPassword",
        "DelayOpen",
    ],
    CustomOptionPolicy::Reject,
    CustomOptionPolicy::Accept,
);

/// SOCKS-IRC has the same listener and proxy-authentication surface as SOCKS;
/// the IRC filter is a payload policy, not a second option namespace.
pub const SOCKS_IRC_OPTIONS: OptionCapabilities = SOCKS_OPTIONS;

/// Streamr consumers require a producer destination and an administrator-selected
/// local UDP target port. `ListenPort`, when present, is the fixed I2P source
/// port used in the Streamr control datagrams.
pub const STREAMR_CLIENT_OPTIONS: OptionCapabilities = OptionCapabilities::new(
    &["TargetPort"],
    &["TargetDestination", "StreamrTarget"],
    &["ListenInterface", "ListenPort", "HostingDestination"],
    CustomOptionPolicy::Reject,
    CustomOptionPolicy::Reject,
);

/// Streamr producers require a local UDP source port. `TargetPort` is the
/// configured I2P destination port used when fanning out payloads.
pub const STREAMR_SERVER_OPTIONS: OptionCapabilities = OptionCapabilities::new(
    &["ListenPort"],
    &[],
    &["TargetPort", "ListenInterface", "HostingDestination"],
    CustomOptionPolicy::Reject,
    CustomOptionPolicy::Reject,
);

fn present_runtime_fields(options: &TunnelOptions) -> Vec<&'static str> {
    let mut fields = Vec::new();
    for (field, present) in [
        ("TargetDestination", options.target_destination.is_some()),
        ("TargetPort", options.target_port.is_some()),
        ("ListenInterface", options.listen_interface.is_some()),
        ("ListenPort", options.listen_port.is_some()),
        ("AccessList", options.access_list.is_some()),
        ("AllowPlaintext", options.allowplaintext.is_some()),
        ("Shared", options.shared.is_some()),
        ("DelayOpen", options.delay_open.is_some()),
        ("UseSSL", options.use_ssl.is_some()),
        ("TunnelLength", options.tunnel_length.is_some()),
        ("TunnelVariance", options.tunnel_variance.is_some()),
        ("TunnelQuantity", options.tunnel_quantity.is_some()),
        (
            "TunnelBackupQuantity",
            options.tunnel_backup_quantity.is_some(),
        ),
        ("SigType", options.sig_type.is_some()),
        ("EncType", options.enc_type.is_some()),
        ("NewDest", options.new_dest.is_some()),
        (
            "PersistentClientKey",
            options.persistent_client_key.is_some(),
        ),
        ("PrivKeyFile", options.priv_key_file.is_some()),
        ("HostingDestination", options.hosting_destination.is_some()),
        ("IsPrivate", options.is_private.is_some()),
        ("HashCash", options.hashcash_proofs_required.is_some()),
        ("SignatureType", options.signature_type.is_some()),
        ("Consumer", options.consumer.is_some()),
        ("SSLCertificate", options.ssl_certificate.is_some()),
        ("SSLKey", options.ssl_key.is_some()),
        ("HttpHost", options.http_host.is_some()),
        ("ProxyUsername", options.proxy_username.is_some()),
        ("ProxyPassword", options.proxy_password.is_some()),
        ("IrcServer", options.irc_server.is_some()),
        ("IrcPort", options.irc_port.is_some()),
        ("IrcNick", options.irc_nick.is_some()),
        ("IrcPassword", options.irc_password.is_some()),
        ("IrcChannels", options.irc_channels.is_some()),
        ("StreamrTarget", options.streamr_target.is_some()),
    ] {
        if present {
            fields.push(field);
        }
    }
    fields
}

fn field_present(options: &TunnelOptions, field: &str) -> bool {
    present_runtime_fields(options).contains(&field)
}

fn valid_custom_options(options: &std::collections::BTreeMap<String, String>) -> bool {
    if options.len() > MAX_CUSTOM_OPTIONS {
        return false;
    }

    let mut folded_keys = std::collections::BTreeSet::new();
    options.iter().all(|(key, value)| {
        key.starts_with("i2cp.")
            && key.len() <= MAX_CUSTOM_OPTION_KEY_LENGTH
            && value.len() <= MAX_CUSTOM_OPTION_VALUE_LENGTH
            && folded_keys.insert(key.to_ascii_lowercase())
            && SessionOption::new(key.clone(), value.clone()).is_ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_and_optional_fields_are_classified() {
        let mut options = TunnelOptions {
            target_destination: Some("public-destination".to_owned()),
            listen_port: Some(0),
            ..Default::default()
        };
        assert!(validate_options(TunnelType::Client, &options, CLIENT_OPTIONS).is_ok());

        options.target_destination = None;
        let error = validate_options(TunnelType::Client, &options, CLIENT_OPTIONS).unwrap_err();
        assert_eq!(
            error.to_string(),
            "client requires option TargetDestination"
        );
    }

    #[test]
    fn delay_open_is_supported_only_by_tcp_client_families() {
        for (tunnel_type, capabilities) in [
            (TunnelType::Client, CLIENT_OPTIONS),
            (TunnelType::HttpClient, HTTP_CLIENT_OPTIONS),
            (TunnelType::IrcClient, IRC_CLIENT_OPTIONS),
            (TunnelType::Socks, SOCKS_OPTIONS),
            (TunnelType::SocksIrc, SOCKS_IRC_OPTIONS),
            (TunnelType::ConnectClient, CONNECT_CLIENT_OPTIONS),
        ] {
            let options = TunnelOptions {
                delay_open: Some(true),
                target_destination: matches!(tunnel_type, TunnelType::Client | TunnelType::IrcClient)
                    .then(|| "destination".to_owned()),
                listen_port: Some(0),
                ..Default::default()
            };
            assert!(validate_options(tunnel_type, &options, capabilities).is_ok());
        }
        let options = TunnelOptions {
            delay_open: Some(true),
            target_destination: Some("destination".to_owned()),
            target_port: Some(80),
            ..Default::default()
        };
        let error = validate_options(TunnelType::StreamrClient, &options, STREAMR_CLIENT_OPTIONS)
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "streamrclient does not support option DelayOpen"
        );
    }

    #[test]
    fn recognized_but_unimplemented_security_option_is_rejected_without_value() {
        let options = TunnelOptions {
            target_destination: Some("public-destination".to_owned()),
            listen_port: Some(0),
            ssl_key: crate::i2pcontrol::domain::tunnel::OptionRedacted::new("secret-key"),
            ..Default::default()
        };

        let error = validate_options(TunnelType::Client, &options, CLIENT_OPTIONS).unwrap_err();
        assert_eq!(error.to_string(), "client does not support option SSLKey");
        assert!(!error.to_string().contains("secret-key"));
    }

    #[test]
    fn new_dest_is_rejected_for_non_streaming_client_families() {
        for tunnel_type in [
            TunnelType::StreamrClient,
            TunnelType::Server,
            TunnelType::HttpServer,
            TunnelType::HttpBidirServer,
            TunnelType::IrcServer,
            TunnelType::StreamrServer,
        ] {
            let options = TunnelOptions {
                new_dest: Some(true),
                ..Default::default()
            };
            let error = validate_common_options(tunnel_type, &options).unwrap_err();
            assert_eq!(error.to_string(), format!("{tunnel_type} does not support option NewDest"));
        }

        for tunnel_type in [
            TunnelType::Client,
            TunnelType::HttpClient,
            TunnelType::IrcClient,
            TunnelType::Socks,
            TunnelType::SocksIrc,
            TunnelType::ConnectClient,
        ] {
            let options = TunnelOptions {
                new_dest: Some(true),
                ..Default::default()
            };
            assert!(validate_common_options(tunnel_type, &options).is_ok());
        }
    }

    #[test]
    fn custom_namespace_policy_is_deterministic() {
        let mut options = TunnelOptions::default();
        options.custom_options.insert("future".to_owned(), "secret".to_owned());
        let capabilities = OptionCapabilities::new(
            &[],
            &[],
            &[],
            CustomOptionPolicy::Accept,
            CustomOptionPolicy::Reject,
        );
        let error = validate_options(TunnelType::HttpServer, &options, capabilities).unwrap_err();
        assert_eq!(
            error.to_string(),
            "httpserver does not support option CustomOptions"
        );
        assert!(!error.to_string().contains("secret"));
    }

    #[test]
    fn session_wire_values_are_strictly_bounded_and_router_supported() {
        let mut options = TunnelOptions {
            tunnel_variance: Some(3),
            ..Default::default()
        };
        assert_eq!(
            validate_common_options(TunnelType::Client, &options)
                .unwrap_err()
                .to_string(),
            "client does not support option TunnelVariance"
        );

        options.tunnel_variance = None;
        options.tunnel_backup_quantity = Some(4);
        assert_eq!(
            validate_common_options(TunnelType::Client, &options)
                .unwrap_err()
                .to_string(),
            "client does not support option TunnelBackupQuantity"
        );

        options.tunnel_backup_quantity = None;
        options.sig_type = Some("11".to_owned());
        assert_eq!(
            validate_common_options(TunnelType::Client, &options)
                .unwrap_err()
                .to_string(),
            "client does not support option SigType"
        );

        options.sig_type = Some("7".to_owned());
        assert!(validate_common_options(TunnelType::Client, &options).is_ok());
    }

    #[test]
    fn custom_options_are_bounded_namespaced_and_cannot_override_typed_fields() {
        let mut options = TunnelOptions::default();
        options
            .custom_options
            .insert("i2cp.custom".to_owned(), "safe-value".to_owned());
        assert!(validate_common_options(TunnelType::Client, &options).is_ok());

        options.custom_options.insert("custom".to_owned(), "value".to_owned());
        assert!(validate_common_options(TunnelType::Client, &options).is_err());

        options.custom_options.remove("custom");
        options
            .custom_options
            .insert("i2cp.leaseSetEncType".to_owned(), "6,4".to_owned());
        assert!(validate_common_options(TunnelType::Client, &options).is_err());

        options.custom_options.clear();
        options
            .custom_options
            .insert("i2cp.custom".to_owned(), "bad value".to_owned());
        assert!(validate_common_options(TunnelType::Client, &options).is_err());
    }
}
