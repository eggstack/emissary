//! Backend-local capability validation for persisted Proposal 170 options.
//!
//! This is intentionally a small declaration/validator pair rather than a
//! global feature matrix. A backend must explicitly classify every
//! runtime-relevant option it accepts; management metadata remains outside
//! the runtime capability boundary.

use std::fmt;

use crate::i2pcontrol::domain::tunnel::{TunnelOptions, TunnelType};

/// Policy for the protocol-defined custom option namespace.
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

/// Runtime fields supported by the existing generic client backend.
pub const CLIENT_OPTIONS: OptionCapabilities = OptionCapabilities::new(
    &["TargetDestination", "ListenPort"],
    &[],
    &[
        "TargetPort",
        "ListenInterface",
        "AccessList",
        "AllowPlaintext",
    ],
    CustomOptionPolicy::Accept,
    CustomOptionPolicy::Accept,
);

/// Runtime fields supported by the existing generic server backend.
pub const SERVER_OPTIONS: OptionCapabilities = OptionCapabilities::new(
    &[],
    &["TargetPort", "ListenPort"],
    &[
        "HostingDestination",
        "IsPrivate",
        "HashCash",
        "SignatureType",
        "Consumer",
    ],
    CustomOptionPolicy::Accept,
    CustomOptionPolicy::Accept,
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
}
