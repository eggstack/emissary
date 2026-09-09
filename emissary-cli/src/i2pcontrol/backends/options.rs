//! Backend-local capability validation for persisted Proposal 170 options.
//!
//! This is intentionally a small declaration/validator pair rather than a
//! global feature matrix. A backend must explicitly classify every
//! runtime-relevant option it accepts; management metadata remains outside
//! the runtime capability boundary.

use std::fmt;

use crate::i2pcontrol::domain::tunnel::{EncryptLeaseSetMode, TunnelOptions, TunnelType};
use yosemite_i2pcontrol::SessionOption;

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
    // connection, so it is deliberately not mapped here. M144 applies UseSSL
    // for the four pinned HTTP/CONNECT families (listener TLS for
    // httpclient/connectclient, loopback-target TLS for
    // httpserver/httpbidirserver); all other families stay not applicable.
    if let Some(_use_ssl) = options.use_ssl {
        if !matches!(
            tunnel_type,
            TunnelType::HttpClient
                | TunnelType::ConnectClient
                | TunnelType::HttpServer
                | TunnelType::HttpBidirServer
        ) {
            return Err(common_unsupported(tunnel_type, "UseSSL"));
        }
    }
    for (present, field) in [(
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
    )] {
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
    // M121 Outcome C: SigType is demoted to blocked_primitive for all tunnel
    // families. Emissary only generates/signs Ed25519 (type 7) end-to-end;
    // configurable semantics require values it cannot produce. Any supplied
    // value — including "7" — fails before allocation; there is no fallback.
    if options.sig_type.is_some() {
        return Err(common_unsupported(tunnel_type, "SigType"));
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
    // M134 (rebased on M137): `NewDest` is applied for the six non-Streamr
    // TCP client families as a proven idle-resume policy. Reference
    // `newDestOnResume` allocates exactly one transient successor only on
    // resume after an actual I2P-session idle close; manual Stop/Start,
    // Restart, failure and unrelated edits never rotate. `NewDest=true`
    // requires `Close=true` (checked where raw config is available:
    // `client_lifecycle_config` / session gates), conflicts with
    // `PersistentClientKey=true` and any `PrivKeyFile` import, and stays
    // `not_applicable` for Streamr and servers. `NewDest=false` is an
    // explicit disabled value for the six families (no prerequisites).
    if let Some(new_dest) = options.new_dest {
        let applicable = matches!(
            tunnel_type,
            TunnelType::Client
                | TunnelType::HttpClient
                | TunnelType::IrcClient
                | TunnelType::Socks
                | TunnelType::SocksIrc
                | TunnelType::ConnectClient
        );
        if !applicable {
            return Err(common_unsupported(tunnel_type, "NewDest"));
        }
        if new_dest && options.persistent_client_key.unwrap_or(false) {
            return Err(common_unsupported(tunnel_type, "NewDest"));
        }
        if new_dest && options.priv_key_file.is_some() {
            return Err(common_unsupported(tunnel_type, "NewDest"));
        }
    }
    // PersistentClientKey and Shared are meaningful only for the control-plane
    // client families and are applied by the bounded owner.
    if options.shared.is_some() && !tunnel_type.is_client() {
        return Err(common_unsupported(tunnel_type, "Shared"));
    }
    if options.persistent_client_key.is_some() && !tunnel_type.is_client() {
        return Err(common_unsupported(tunnel_type, "PersistentClientKey"));
    }

    // M162: LeaseSet-security fields stay blocked before allocation on every
    // family (servers blocked_primitive, clients not_applicable). No mode is
    // accepted inertly and no secret is echoed.
    validate_lease_set_security(tunnel_type, options)?;

    Ok(())
}

/// Whether an `OptionalLookup` secret is required, forbidden, or absent for
/// one exact `EncryptLeaseSet` mode (M162 ten-mode table).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseSetLookupRequirement {
    Required,
    Forbidden,
    Absent,
}

/// Whether indexed `LeaseSetClientAuths` entries are required, allowed, or
/// forbidden for one exact mode (M162 ten-mode table).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LeaseSetIndexedRequirement {
    Required,
    Allowed,
    Forbidden,
}

/// One executable row of the M162 ten-mode machine table.
///
/// This is acceptance authority, not a runtime capability source: every row
/// currently records a blocked disposition (M161-B legacy plus Yosemite
/// base-key/duplicate/bound gaps). Tests iterate this table across all five
/// server families and assert fail-before-allocation with redaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaseSetSecurityModeRow {
    /// Exact Proposal `EncryptLeaseSet` spelling.
    pub mode: EncryptLeaseSetMode,
    /// Legacy `i2cp.encryptLeaseSet=true` selector (true) or absent (false).
    pub legacy_encrypt_flag: bool,
    /// Modern `i2cp.leaseSetType` selector when present.
    pub lease_set_type: Option<u8>,
    /// `i2cp.leaseSetAuthType` selector when present.
    pub auth_type: Option<u8>,
    /// Neutral lower primitive that would own the mode.
    pub primitive: &'static str,
    /// Lookup-secret coupling.
    pub lookup: LeaseSetLookupRequirement,
    /// Indexed per-user coupling.
    pub indexed: LeaseSetIndexedRequirement,
    /// Key interpretation (`none`, `psk-32B`, `dh-x25519-32B`).
    pub key_kind: &'static str,
    /// Persistent base-key role.
    pub base_role: &'static str,
    /// Extended-B32 `secret_required` flag when the mode were operational.
    pub b32_secret_required: bool,
    /// Extended-B32 `auth_required` flag when the mode were operational.
    pub b32_auth_required: bool,
    /// Generation/import rule (future custody design, currently unallocated).
    pub generation: &'static str,
    /// Published DatabaseStore type when the mode were operational.
    pub database_store: &'static str,
    /// Whether M161 legacy-AES disposition gates this mode.
    pub m161_dependency: bool,
    /// M162 blocked reason (executable: every row is blocked).
    pub blocked_reason: &'static str,
}

/// Machine-readable ten-mode table in Proposal wire order (M162 §Ten-mode).
///
/// Order matches `ALL_ENCRYPT_LEASE_SET_MODES`: disable, encrypted (aes),
/// blinded, blinded+lookup, psk, psk+lookup, psk per-user, psk lookup+per-user,
/// dh per-user, dh lookup+per-user.
pub const LEASE_SET_SECURITY_MODES: &[LeaseSetSecurityModeRow] = &[
    LeaseSetSecurityModeRow {
        mode: EncryptLeaseSetMode::Disable,
        legacy_encrypt_flag: false,
        lease_set_type: None,
        auth_type: None,
        primitive: "none/ordinary",
        lookup: LeaseSetLookupRequirement::Forbidden,
        indexed: LeaseSetIndexedRequirement::Forbidden,
        key_kind: "none",
        base_role: "none; omit field for ordinary type-3 publication",
        b32_secret_required: false,
        b32_auth_required: false,
        generation: "none; no secret custody",
        database_store: "ordinary LeaseSet2 type 3 (field omitted)",
        m161_dependency: false,
        blocked_reason:
            "M162 blocked: explicit disable is rejected before allocation; omit the field for ordinary publication (matrix stays blocked_primitive)",
    },
    LeaseSetSecurityModeRow {
        mode: EncryptLeaseSetMode::EncryptedAes,
        legacy_encrypt_flag: true,
        lease_set_type: None,
        auth_type: None,
        primitive: "M161 legacy LS1 AES",
        lookup: LeaseSetLookupRequirement::Absent,
        indexed: LeaseSetIndexedRequirement::Forbidden,
        key_kind: "none (legacy SessionKey/keyring, no modern base)",
        base_role: "none modern; legacy destination-hash keyring (unimplemented)",
        b32_secret_required: false,
        b32_auth_required: false,
        generation: "none; LS1 resurrection forbidden inside M162",
        database_store: "legacy LeaseSet type 1 with keyring-gated AES leases (unimplemented)",
        m161_dependency: true,
        blocked_reason:
            "M161 outcome B: valid but blocked legacy LS1 (deprecated, insecure); all five EncryptLeaseSet cells stay blocked",
    },
    LeaseSetSecurityModeRow {
        mode: EncryptLeaseSetMode::Blinded,
        legacy_encrypt_flag: false,
        lease_set_type: Some(5),
        auth_type: Some(0),
        primitive: "M157 no-auth type-5",
        lookup: LeaseSetLookupRequirement::Forbidden,
        indexed: LeaseSetIndexedRequirement::Forbidden,
        key_kind: "none",
        base_role: "none; blinding is core-internal (M156), no I2PControl base",
        b32_secret_required: false,
        b32_auth_required: false,
        generation: "none I2PControl custody; core blinding only",
        database_store: "EncryptedLeaseSet2 type 5, blinded DHT key (frozen M157)",
        m161_dependency: false,
        blocked_reason:
            "M162 blocked: Yosemite-expressible but field-level blocked pending full ten-value domain (no partial-enum apply)",
    },
    LeaseSetSecurityModeRow {
        mode: EncryptLeaseSetMode::BlindedWithLookup,
        legacy_encrypt_flag: false,
        lease_set_type: Some(5),
        auth_type: Some(0),
        primitive: "M158 lookup-secret type-5",
        lookup: LeaseSetLookupRequirement::Required,
        indexed: LeaseSetIndexedRequirement::Forbidden,
        key_kind: "none",
        base_role: "none; lookup secret imported from OptionalLookup only",
        b32_secret_required: true,
        b32_auth_required: false,
        generation: "import lookup Base64(UTF8) persistently; no base key",
        database_store: "EncryptedLeaseSet2 type 5, blinded DHT key",
        m161_dependency: false,
        blocked_reason:
            "M162 blocked: lookup custody/transaction owner absent; field-level blocked pending full domain",
    },
    LeaseSetSecurityModeRow {
        mode: EncryptLeaseSetMode::EncryptedPsk,
        legacy_encrypt_flag: false,
        lease_set_type: Some(5),
        auth_type: Some(2),
        primitive: "M159 PSK base-only",
        lookup: LeaseSetLookupRequirement::Forbidden,
        indexed: LeaseSetIndexedRequirement::Forbidden,
        key_kind: "psk-32B base",
        base_role: "generated persistent 32B base PSK; authorized even with zero indexed",
        b32_secret_required: false,
        b32_auth_required: true,
        generation: "generate base PSK persistently; no import surface in Proposal",
        database_store: "EncryptedLeaseSet2 type 5, blinded DHT key",
        m161_dependency: false,
        blocked_reason:
            "M162 blocked: Yosemite 59140a2 has no typed leaseSetPrivKey base emission (reserved generic); base would be inert",
    },
    LeaseSetSecurityModeRow {
        mode: EncryptLeaseSetMode::EncryptedWithLookupPsk,
        legacy_encrypt_flag: false,
        lease_set_type: Some(5),
        auth_type: Some(2),
        primitive: "M159 PSK + M158 secret",
        lookup: LeaseSetLookupRequirement::Required,
        indexed: LeaseSetIndexedRequirement::Forbidden,
        key_kind: "psk-32B base",
        base_role: "generated persistent base PSK plus imported lookup secret",
        b32_secret_required: true,
        b32_auth_required: true,
        generation: "import lookup plus generate base PSK persistently",
        database_store: "EncryptedLeaseSet2 type 5, blinded DHT key",
        m161_dependency: false,
        blocked_reason: "M162 blocked: Yosemite base-key gap plus lookup custody absent",
    },
    LeaseSetSecurityModeRow {
        mode: EncryptLeaseSetMode::EncryptedPerUserPsk,
        legacy_encrypt_flag: false,
        lease_set_type: Some(5),
        auth_type: Some(2),
        primitive: "M159 PSK per-user",
        lookup: LeaseSetLookupRequirement::Forbidden,
        indexed: LeaseSetIndexedRequirement::Required,
        key_kind: "psk-32B base + indexed",
        base_role: "generated base PSK plus imported indexed per-user PSKs",
        b32_secret_required: false,
        b32_auth_required: true,
        generation: "generate base plus import indexed {name,key} persistently",
        database_store: "EncryptedLeaseSet2 type 5, blinded DHT key",
        m161_dependency: false,
        blocked_reason:
            "M162 blocked: Yosemite base gap plus duplicate/bound gaps (Yosemite rejects duplicates, caps 16 vs core 99)",
    },
    LeaseSetSecurityModeRow {
        mode: EncryptLeaseSetMode::EncryptedLookupPerUserPsk,
        legacy_encrypt_flag: false,
        lease_set_type: Some(5),
        auth_type: Some(2),
        primitive: "M159 PSK per-user + M158 secret",
        lookup: LeaseSetLookupRequirement::Required,
        indexed: LeaseSetIndexedRequirement::Required,
        key_kind: "psk-32B base + indexed",
        base_role: "generated base PSK plus imported lookup and indexed PSKs",
        b32_secret_required: true,
        b32_auth_required: true,
        generation: "import lookup plus generate base plus import indexed",
        database_store: "EncryptedLeaseSet2 type 5, blinded DHT key",
        m161_dependency: false,
        blocked_reason: "M162 blocked: Yosemite base/duplicate/bound gaps plus lookup custody absent",
    },
    LeaseSetSecurityModeRow {
        mode: EncryptLeaseSetMode::EncryptedPerUserDh,
        legacy_encrypt_flag: false,
        lease_set_type: Some(5),
        auth_type: Some(1),
        primitive: "M160 DH per-user",
        lookup: LeaseSetLookupRequirement::Forbidden,
        indexed: LeaseSetIndexedRequirement::Required,
        key_kind: "dh-x25519-32B (base private + indexed publics)",
        base_role: "generated persistent X25519 private; derived public always authorized plus indexed",
        b32_secret_required: false,
        b32_auth_required: true,
        generation: "generate base X25519 private persistently plus import indexed publics",
        database_store: "EncryptedLeaseSet2 type 5, blinded DHT key",
        m161_dependency: false,
        blocked_reason:
            "M162 blocked: Yosemite base gap plus duplicate/bound gaps (Yosemite rejects duplicates, caps 16 vs core 99)",
    },
    LeaseSetSecurityModeRow {
        mode: EncryptLeaseSetMode::EncryptedLookupPerUserDh,
        legacy_encrypt_flag: false,
        lease_set_type: Some(5),
        auth_type: Some(1),
        primitive: "M160 DH per-user + M158 secret",
        lookup: LeaseSetLookupRequirement::Required,
        indexed: LeaseSetIndexedRequirement::Required,
        key_kind: "dh-x25519-32B (base private + indexed publics)",
        base_role: "generated base X25519 private plus imported lookup and indexed publics",
        b32_secret_required: true,
        b32_auth_required: true,
        generation: "import lookup plus generate base private plus import indexed",
        database_store: "EncryptedLeaseSet2 type 5, blinded DHT key",
        m161_dependency: false,
        blocked_reason: "M162 blocked: Yosemite base/duplicate/bound gaps plus lookup custody absent",
    },
];

/// Fail-before-allocation gate for all LeaseSet-security typed fields.
///
/// M162 keeps every mode blocked (including explicit `disable`: omit the
/// field for ordinary publication). Any typed presence — mode selector,
/// lookup secret, or per-user entries — rejects with the field name only;
/// secret values never enter the error. This is the common gate every server
/// backend reaches through `validate_common_options` before any secret-store
/// lookup, runtime reservation, or SAM wire work.
pub fn validate_lease_set_security(
    tunnel_type: TunnelType,
    options: &TunnelOptions,
) -> Result<(), OptionValidationError> {
    // Executable-table anchor: the ten-mode authority stays exactly ten rows
    // in Proposal wire order. Every row is currently blocked (§M162).
    debug_assert_eq!(LEASE_SET_SECURITY_MODES.len(), 10);
    debug_assert_eq!(
        LEASE_SET_SECURITY_MODES[0].mode,
        EncryptLeaseSetMode::Disable
    );
    if let Some(mode) = options.encrypt_lease_set {
        let _ = mode;
        return Err(common_unsupported(tunnel_type, "EncryptLeaseSet"));
    }
    if options.optional_lookup.is_some() {
        return Err(common_unsupported(tunnel_type, "OptionalLookup"));
    }
    if !options.lease_set_client_auths.is_empty() {
        return Err(common_unsupported(tunnel_type, "LeaseSetClientAuths"));
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
            | "EncryptLeaseSet"
            | "OptionalLookup"
            | "LeaseSetClientAuths"
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
///
/// `HostingDestination` is the control-plane-persisted public destination
/// display field written after a successful start. All other server families
/// already accept it; the generic server must as well or a committed
/// definition could never restart. It carries no Proposal matrix cell.
pub const SERVER_OPTIONS: OptionCapabilities = OptionCapabilities::new(
    &[],
    &["TargetPort", "ListenPort"],
    &["TargetHost", "Host", "HostingDestination"],
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
        ("EncryptLeaseSet", options.encrypt_lease_set.is_some()),
        ("OptionalLookup", options.optional_lookup.is_some()),
        (
            "LeaseSetClientAuths",
            !options.lease_set_client_auths.is_empty(),
        ),
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
                target_destination: matches!(
                    tunnel_type,
                    TunnelType::Client | TunnelType::IrcClient
                )
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
    fn m134_new_dest_applies_to_six_tcp_families_with_conflicts() {
        // M134 (rebased on M137): `NewDest` is applied for the six non-Streamr
        // TCP clients. `NewDest=true` conflicts with persistent/import identity;
        // `NewDest=false` is explicit disabled. Streamr and servers stay N/A.
        for tunnel_type in [
            TunnelType::Client,
            TunnelType::HttpClient,
            TunnelType::IrcClient,
            TunnelType::Socks,
            TunnelType::SocksIrc,
            TunnelType::ConnectClient,
        ] {
            for value in [true, false] {
                let options = TunnelOptions {
                    new_dest: Some(value),
                    ..Default::default()
                };
                assert!(
                    validate_common_options(tunnel_type, &options).is_ok(),
                    "{tunnel_type} NewDest={value} must validate (Close prerequisite is enforced where raw config is available)"
                );
            }
            // `NewDest=true` + persistent conflicts before allocation.
            let options = TunnelOptions {
                new_dest: Some(true),
                persistent_client_key: Some(true),
                ..Default::default()
            };
            assert_eq!(
                validate_common_options(tunnel_type, &options).unwrap_err().to_string(),
                format!("{tunnel_type} does not support option NewDest")
            );
            // `NewDest=true` + import conflicts before allocation.
            let options = TunnelOptions {
                new_dest: Some(true),
                priv_key_file: Some("import.key".to_owned()),
                ..Default::default()
            };
            assert_eq!(
                validate_common_options(tunnel_type, &options).unwrap_err().to_string(),
                format!("{tunnel_type} does not support option NewDest")
            );
            // `NewDest=false` + persistent/import stays compatible (disabled).
            let options = TunnelOptions {
                new_dest: Some(false),
                persistent_client_key: Some(true),
                ..Default::default()
            };
            assert!(validate_common_options(tunnel_type, &options).is_ok());
        }
        for tunnel_type in [
            TunnelType::StreamrClient,
            TunnelType::Server,
            TunnelType::HttpServer,
            TunnelType::HttpBidirServer,
            TunnelType::IrcServer,
            TunnelType::StreamrServer,
        ] {
            for value in [true, false] {
                let options = TunnelOptions {
                    new_dest: Some(value),
                    ..Default::default()
                };
                let error = validate_common_options(tunnel_type, &options).unwrap_err();
                assert_eq!(
                    error.to_string(),
                    format!("{tunnel_type} does not support option NewDest")
                );
            }
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
            validate_common_options(TunnelType::Client, &options).unwrap_err().to_string(),
            "client does not support option TunnelVariance"
        );

        options.tunnel_variance = None;
        options.tunnel_backup_quantity = Some(4);
        assert_eq!(
            validate_common_options(TunnelType::Client, &options).unwrap_err().to_string(),
            "client does not support option TunnelBackupQuantity"
        );

        options.tunnel_backup_quantity = None;
        options.sig_type = Some("11".to_owned());
        assert_eq!(
            validate_common_options(TunnelType::Client, &options).unwrap_err().to_string(),
            "client does not support option SigType"
        );

        // M121 Outcome C: even the router-native "7" is blocked as a
        // configurable Proposal option. Emissary cannot generate/sign any
        // other type end-to-end, so a singleton domain is inert, not support.
        for value in ["7", "07", " 7", "EdDSA_SHA512_Ed25519", "1", "0", "11"] {
            options.sig_type = Some(value.to_owned());
            let error = validate_common_options(TunnelType::Client, &options).unwrap_err();
            assert_eq!(
                error.to_string(),
                "client does not support option SigType",
                "SigType value {value:?} must fail before allocation"
            );
            assert!(
                !error.to_string().contains(value.trim()),
                "rejection must not echo the value"
            );
        }
        options.sig_type = Some(String::new());
        assert_eq!(
            validate_common_options(TunnelType::Client, &options).unwrap_err().to_string(),
            "client does not support option SigType"
        );
    }

    #[test]
    fn m121_sigtype_is_blocked_for_all_applicable_families_without_fallback() {
        for tunnel_type in [
            TunnelType::Client,
            TunnelType::HttpClient,
            TunnelType::IrcClient,
            TunnelType::Socks,
            TunnelType::SocksIrc,
            TunnelType::ConnectClient,
            TunnelType::Server,
            TunnelType::HttpServer,
            TunnelType::HttpBidirServer,
            TunnelType::IrcServer,
        ] {
            let options = TunnelOptions {
                sig_type: Some("7".to_owned()),
                ..Default::default()
            };
            let error = validate_common_options(tunnel_type, &options).unwrap_err();
            assert_eq!(
                error.to_string(),
                format!("{tunnel_type} does not support option SigType")
            );
        }
    }

    #[test]
    fn custom_options_are_bounded_namespaced_and_cannot_override_typed_fields() {
        let mut options = TunnelOptions::default();
        options.custom_options.insert("i2cp.custom".to_owned(), "safe-value".to_owned());
        assert!(validate_common_options(TunnelType::Client, &options).is_ok());

        options.custom_options.insert("custom".to_owned(), "value".to_owned());
        assert!(validate_common_options(TunnelType::Client, &options).is_err());

        options.custom_options.remove("custom");
        options
            .custom_options
            .insert("i2cp.leaseSetEncType".to_owned(), "6,4".to_owned());
        assert!(validate_common_options(TunnelType::Client, &options).is_err());

        options.custom_options.clear();
        options.custom_options.insert("i2cp.custom".to_owned(), "bad value".to_owned());
        assert!(validate_common_options(TunnelType::Client, &options).is_err());
    }

    #[test]
    fn m162_ten_mode_table_is_exact_and_blocked() {
        use crate::i2pcontrol::domain::tunnel::EncryptLeaseSetMode;
        assert_eq!(LEASE_SET_SECURITY_MODES.len(), 10);
        let spellings = [
            "disable",
            "encrypted (aes)",
            "blinded",
            "blinded with lookup password",
            "encrypted (psk)",
            "encrypted with lookup password (psk)",
            "encrypted with per-user key (psk)",
            "encrypted with lookup password and per-user key (psk)",
            "encrypted with per-user key (dh)",
            "encrypted with lookup password and per-user key (dh)",
        ];
        for (row, spelling) in LEASE_SET_SECURITY_MODES.iter().zip(spellings) {
            assert_eq!(row.mode.as_str(), spelling);
            assert!(!row.blocked_reason.is_empty());
            assert!(row.blocked_reason.contains("blocked"));
        }
        // Legacy/modern selector split: only legacy AES sets the legacy flag;
        // every modern mode selects type 5 and never the legacy flag.
        assert!(LEASE_SET_SECURITY_MODES[1].legacy_encrypt_flag);
        assert!(LEASE_SET_SECURITY_MODES[1].lease_set_type.is_none());
        assert!(LEASE_SET_SECURITY_MODES[1].m161_dependency);
        for row in &LEASE_SET_SECURITY_MODES[2..] {
            assert!(!row.legacy_encrypt_flag);
            assert_eq!(row.lease_set_type, Some(5));
            assert!(!row.m161_dependency);
        }
        // Auth selectors: disable none, legacy none, blinded 0, PSK 2, DH 1.
        assert_eq!(LEASE_SET_SECURITY_MODES[0].auth_type, None);
        assert_eq!(LEASE_SET_SECURITY_MODES[1].auth_type, None);
        assert_eq!(LEASE_SET_SECURITY_MODES[2].auth_type, Some(0));
        assert_eq!(LEASE_SET_SECURITY_MODES[3].auth_type, Some(0));
        for row in &LEASE_SET_SECURITY_MODES[4..8] {
            assert_eq!(row.auth_type, Some(2));
            assert!(row.key_kind.contains("psk"));
        }
        for row in &LEASE_SET_SECURITY_MODES[8..] {
            assert_eq!(row.auth_type, Some(1));
            assert!(row.key_kind.contains("dh"));
        }
        // Lookup/indexed coupling.
        assert_eq!(
            LEASE_SET_SECURITY_MODES[3].lookup,
            LeaseSetLookupRequirement::Required
        );
        assert_eq!(
            LEASE_SET_SECURITY_MODES[2].lookup,
            LeaseSetLookupRequirement::Forbidden
        );
        assert_eq!(
            LEASE_SET_SECURITY_MODES[6].indexed,
            LeaseSetIndexedRequirement::Required
        );
        assert_eq!(
            LEASE_SET_SECURITY_MODES[4].indexed,
            LeaseSetIndexedRequirement::Forbidden
        );
        // B32 flags follow secret/auth presence.
        assert!(!LEASE_SET_SECURITY_MODES[2].b32_secret_required);
        assert!(LEASE_SET_SECURITY_MODES[3].b32_secret_required);
        assert!(LEASE_SET_SECURITY_MODES[4].b32_auth_required);
        assert!(!LEASE_SET_SECURITY_MODES[0].b32_auth_required);
        // Modern rows publish type-5 ELS2; legacy publishes LS1; disable is ordinary.
        assert!(LEASE_SET_SECURITY_MODES[2].database_store.contains("type 5"));
        assert!(LEASE_SET_SECURITY_MODES[1].database_store.contains("type 1"));
        assert!(LEASE_SET_SECURITY_MODES[0].database_store.contains("type 3"));
        let _ = EncryptLeaseSetMode::Disable;
    }

    #[test]
    fn m162_all_typed_leaseset_presence_fails_before_allocation_without_echo() {
        use crate::i2pcontrol::domain::tunnel::{LeaseSetClientAuthEntry, OptionRedacted};
        // Every server family plus a client family: any typed presence rejects
        // with the field name only.
        for tunnel_type in [
            TunnelType::Server,
            TunnelType::HttpServer,
            TunnelType::HttpBidirServer,
            TunnelType::IrcServer,
            TunnelType::StreamrServer,
            TunnelType::Client,
        ] {
            for mode in crate::i2pcontrol::domain::tunnel::ALL_ENCRYPT_LEASE_SET_MODES {
                let options = TunnelOptions {
                    encrypt_lease_set: Some(*mode),
                    ..Default::default()
                };
                let error = validate_common_options(tunnel_type, &options).unwrap_err();
                assert_eq!(
                    error.to_string(),
                    format!("{tunnel_type} does not support option EncryptLeaseSet"),
                    "mode {mode} on {tunnel_type} must fail before allocation"
                );
                assert!(!error.to_string().contains(mode.as_str()));
            }
            let options = TunnelOptions {
                optional_lookup: OptionRedacted::new("top-secret-lookup"),
                ..Default::default()
            };
            let error = validate_common_options(tunnel_type, &options).unwrap_err();
            assert_eq!(
                error.to_string(),
                format!("{tunnel_type} does not support option OptionalLookup")
            );
            assert!(!error.to_string().contains("top-secret"));

            let options = TunnelOptions {
                lease_set_client_auths: vec![LeaseSetClientAuthEntry::new(
                    "carol",
                    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
                )
                .unwrap()],
                ..Default::default()
            };
            let error = validate_common_options(tunnel_type, &options).unwrap_err();
            assert_eq!(
                error.to_string(),
                format!("{tunnel_type} does not support option LeaseSetClientAuths")
            );
            assert!(!error.to_string().contains("carol"));
            assert!(!error.to_string().contains("AAAA"));
        }
        // Absent fields still validate (ordinary path unaffected).
        assert!(validate_common_options(TunnelType::Server, &TunnelOptions::default()).is_ok());
    }
}
