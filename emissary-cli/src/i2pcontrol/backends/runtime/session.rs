//! Common Proposal 170 session option translation.
//!
//! This module is the single I2PControl-owned boundary between canonical
//! tunnel options and Yosemite's `SessionOptions`. It intentionally exposes
//! no Proposal 170 types to the router or core crates.

use std::{
    collections::BTreeMap,
    sync::Arc,
    time::Duration,
};

use parking_lot::Mutex;
use tokio::sync::{broadcast, mpsc, oneshot, Notify};
use yosemite_i2pcontrol::{style, DatagramOptions, DestinationKind, Session, SessionOptions};

use super::super::{options::validate_common_options, BackendError, BackendResult};
use crate::i2pcontrol::client_secret_store::ClientDestinationStore;
use crate::i2pcontrol::domain::tunnel::{TunnelDefinition, TunnelOptions, TunnelType};

type StreamSession = Arc<Mutex<Session<style::Stream>>>;

const DATAGRAM_BUFFER_SIZE: usize = 4095;
const MAX_CONNECT_DELAY: u64 = 60_000;
const DEFAULT_CLOSE_IDLE_TIME: Duration = Duration::from_secs(30 * 60);

/// Generation-local lifecycle controls for streaming client listeners.
///
/// Only `ConnectDelay` remains an applied Proposal lifecycle effect. `Close`,
/// `CloseTime`, and `NewDest` are M121-demoted to `blocked_primitive`: the
/// reference idle policy observes I2P-session activity while the local owner
/// can only count accepted TCP handler tasks, and Yosemite exposes no
/// session-activity observation primitive. Any supplied close/new-dest value
/// fails before allocation (see `client_lifecycle_config`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct ClientLifecycleConfig {
    pub(crate) connect_delay: Option<Duration>,
    pub(crate) close_on_idle: bool,
    pub(crate) close_idle_time: Duration,
    pub(crate) new_dest_on_resume: bool,
}

impl ClientLifecycleConfig {
    pub(crate) const DISABLED: Self = Self {
        connect_delay: None,
        close_on_idle: false,
        close_idle_time: DEFAULT_CLOSE_IDLE_TIME,
        new_dest_on_resume: false,
    };
}

/// An inbound datagram delivered by the single owner of a shared repliable
/// session.  Subscribers receive independent owned payloads, so no caller
/// can hold a session or bookkeeping lock across network I/O.
#[derive(Clone, Debug)]
pub(crate) struct SharedDatagramEvent {
    pub(crate) payload: Vec<u8>,
    pub(crate) peer: String,
}

enum DatagramCommand {
    Send {
        payload: Vec<u8>,
        destination: String,
        options: DatagramOptions,
        response: oneshot::Sender<Result<(), String>>,
    },
}

/// Actor-like owner for one repliable Yosemite session.
#[derive(Clone)]
pub(crate) struct SharedDatagramSession {
    commands: mpsc::Sender<DatagramCommand>,
    events: broadcast::Sender<SharedDatagramEvent>,
}

impl SharedDatagramSession {
    pub(crate) fn spawn(mut session: Session<style::Repliable>) -> Arc<Self> {
        let (commands, mut command_receiver) = mpsc::channel(64);
        let (events, _) = broadcast::channel(128);
        let owner = Arc::new(Self {
            commands,
            events: events.clone(),
        });
        tokio::spawn(async move {
            let mut buffer = vec![0u8; DATAGRAM_BUFFER_SIZE];
            loop {
                tokio::select! {
                    command = command_receiver.recv() => {
                        let Some(DatagramCommand::Send {
                            payload,
                            destination,
                            options,
                            response,
                        }) = command else {
                            break;
                        };
                        let result = session
                            .send_to_with_options(&payload, &destination, options)
                            .await
                            .map_err(|_| "shared datagram send failed".to_owned());
                        let _ = response.send(result);
                    }
                    result = session.recv_from(&mut buffer) => {
                        let Ok((length, peer)) = result else {
                            break;
                        };
                        let _ = events.send(SharedDatagramEvent {
                            payload: buffer[..length].to_vec(),
                            peer,
                        });
                    }
                }
            }
        });
        owner
    }

    pub(crate) fn subscribe(&self) -> broadcast::Receiver<SharedDatagramEvent> {
        self.events.subscribe()
    }

    pub(crate) async fn send_to_with_options(
        &self,
        payload: &[u8],
        destination: &str,
        options: DatagramOptions,
    ) -> Result<(), String> {
        let (response, receiver) = oneshot::channel();
        self.commands
            .send(DatagramCommand::Send {
                payload: payload.to_vec(),
                destination: destination.to_owned(),
                options,
                response,
            })
            .await
            .map_err(|_| "shared datagram session is closed".to_owned())?;
        receiver
            .await
            .map_err(|_| "shared datagram session is closed".to_owned())?
    }
}

enum SessionHandle {
    Stream(StreamSession),
    Datagram(Arc<SharedDatagramSession>),
}

struct SharedSessionEntry {
    session: Option<SessionHandle>,
    members: usize,
    creating: bool,
    notify: Arc<Notify>,
}

#[derive(Default)]
struct SharedSessionState {
    entries: BTreeMap<CompatibilityKey, SharedSessionEntry>,
}

/// Exact shared-session identity. The persistent key is retained only in this
/// private equality/order authority; its formatting is always redacted.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CompatibilityKey {
    style: String,
    identity: CompatibilityIdentity,
    session_options: String,
    session_options_identity: String,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum CompatibilityIdentity {
    Transient,
    Persistent(String),
}

impl std::fmt::Debug for CompatibilityKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompatibilityKey")
            .field("style", &self.style)
            .field(
                "identity",
                &match &self.identity {
                    CompatibilityIdentity::Transient => "transient",
                    CompatibilityIdentity::Persistent(_) => "persistent",
                },
            )
            .field("session_options", &self.session_options)
            .finish()
    }
}

impl std::fmt::Display for CompatibilityKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}", self.style, self.session_options)
    }
}

/// A per-key creator reservation that reopens the key if the creator future
/// is cancelled, panics, times out, or otherwise drops before publication.
struct CreationReservation {
    registry: SharedClientSessionRegistry,
    key: CompatibilityKey,
    active: bool,
}

impl CreationReservation {
    fn new(registry: SharedClientSessionRegistry, key: CompatibilityKey) -> Self {
        Self {
            registry,
            key,
            active: true,
        }
    }

    fn disarm(&mut self) {
        self.active = false;
    }
}

impl Drop for CreationReservation {
    fn drop(&mut self) {
        if self.active {
            self.registry.cancel_creation(&self.key);
        }
    }
}

/// Bounded owner for I2PControl-shared client sessions.
///
/// The compatibility key contains every translated Yosemite session setting
/// and the exact persistent identity material. Session construction happens
/// outside the bookkeeping lock; the per-key creator reservation prevents
/// concurrent equivalent starts from duplicating owners.
#[derive(Clone)]
pub struct SharedClientSessionRegistry {
    state: Arc<Mutex<SharedSessionState>>,
    max_sessions: usize,
}

impl Default for SharedClientSessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

const MAX_SHARED_SESSIONS: usize = 1000;
const MAX_SHARED_MEMBERS: usize = 1000;

impl std::fmt::Debug for SharedClientSessionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let count = self.state.lock().entries.len();
        f.debug_struct("SharedClientSessionRegistry")
            .field("session_count", &count)
            .field("max_sessions", &self.max_sessions)
            .finish()
    }
}

impl SharedClientSessionRegistry {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SharedSessionState::default())),
            max_sessions: MAX_SHARED_SESSIONS,
        }
    }

    pub async fn acquire_stream(
        &self,
        options: SessionOptions,
    ) -> Result<SharedStreamSessionLease, String> {
        let key = compatibility_key(&options, "stream");
        loop {
            let waiter = {
                let mut state = self.state.lock();
                if let Some(entry) = state.entries.get_mut(&key) {
                    if let Some(SessionHandle::Stream(session)) = &entry.session {
                        if entry.members >= MAX_SHARED_MEMBERS {
                            return Err("shared client session member capacity exhausted".to_string());
                        }
                        entry.members += 1;
                        return Ok(SharedStreamSessionLease {
                            session: Arc::clone(session),
                            registry: self.clone(),
                            key: key.clone(),
                        });
                    }
                    if entry.creating {
                        let mut waiter = Box::pin(Arc::clone(&entry.notify).notified_owned());
                        waiter.as_mut().enable();
                        Some(waiter)
                    } else {
                        entry.creating = true;
                        None
                    }
                } else {
                    if state.entries.len() >= self.max_sessions {
                        return Err("shared client session capacity exhausted".to_string());
                    }
                    let notify = Arc::new(Notify::new());
                    state.entries.insert(
                        key.clone(),
                        SharedSessionEntry {
                            session: None,
                            members: 0,
                            creating: true,
                            notify: Arc::clone(&notify),
                        },
                    );
                    None
                }
            };
            if let Some(waiter) = waiter {
                waiter.await;
                continue;
            }
            let mut reservation = CreationReservation::new(self.clone(), key.clone());
            let result = Session::<style::Stream>::new(options.clone())
                .await
                .map(|session| Arc::new(Mutex::new(session)))
                .map_err(|_| "client shared session setup failed".to_string());
            let mut state = self.state.lock();
            let Some(entry) = state.entries.get_mut(&key) else {
                return Err("client shared session reservation was lost".to_string());
            };
            entry.creating = false;
            match result {
                Ok(session) => {
                    entry.members = 1;
                    entry.session = Some(SessionHandle::Stream(Arc::clone(&session)));
                    entry.notify.notify_waiters();
                    reservation.disarm();
                    return Ok(SharedStreamSessionLease {
                        session,
                        registry: self.clone(),
                        key,
                    });
                }
                Err(error) => {
                    let notify = Arc::clone(&entry.notify);
                    state.entries.remove(&key);
                    notify.notify_waiters();
                    reservation.disarm();
                    return Err(error);
                }
            }
        }
    }

    pub async fn acquire_datagram(
        &self,
        options: SessionOptions,
    ) -> Result<SharedDatagramSessionLease, String> {
        let key = compatibility_key(&options, "datagram");
        loop {
            let waiter = {
                let mut state = self.state.lock();
                if let Some(entry) = state.entries.get_mut(&key) {
                    if let Some(SessionHandle::Datagram(session)) = &entry.session {
                        if entry.members >= MAX_SHARED_MEMBERS {
                            return Err("shared client session member capacity exhausted".to_string());
                        }
                        entry.members += 1;
                        return Ok(SharedDatagramSessionLease {
                            session: Arc::clone(session),
                            registry: self.clone(),
                            key: key.clone(),
                        });
                    }
                    if entry.creating {
                        let mut waiter = Box::pin(Arc::clone(&entry.notify).notified_owned());
                        waiter.as_mut().enable();
                        Some(waiter)
                    } else {
                        entry.creating = true;
                        None
                    }
                } else {
                    if state.entries.len() >= self.max_sessions {
                        return Err("shared client session capacity exhausted".to_string());
                    }
                    let notify = Arc::new(Notify::new());
                    state.entries.insert(
                        key.clone(),
                        SharedSessionEntry {
                            session: None,
                            members: 0,
                            creating: true,
                            notify: Arc::clone(&notify),
                        },
                    );
                    None
                }
            };
            if let Some(waiter) = waiter {
                waiter.await;
                continue;
            }
            let mut reservation = CreationReservation::new(self.clone(), key.clone());
            let result = Session::<style::Repliable>::new(options.clone())
                .await
                .map(SharedDatagramSession::spawn)
                .map_err(|_| "client shared datagram session setup failed".to_string());
            let mut state = self.state.lock();
            let Some(entry) = state.entries.get_mut(&key) else {
                return Err("client shared datagram session reservation was lost".to_string());
            };
            entry.creating = false;
            match result {
                Ok(session) => {
                    entry.members = 1;
                    entry.session = Some(SessionHandle::Datagram(Arc::clone(&session)));
                    entry.notify.notify_waiters();
                    reservation.disarm();
                    return Ok(SharedDatagramSessionLease {
                        session,
                        registry: self.clone(),
                        key,
                    });
                }
                Err(error) => {
                    let notify = Arc::clone(&entry.notify);
                    state.entries.remove(&key);
                    notify.notify_waiters();
                    reservation.disarm();
                    return Err(error);
                }
            }
        }
    }

    fn cancel_creation(&self, key: &CompatibilityKey) {
        let notify = {
            let mut state = self.state.lock();
            let Some(entry) = state.entries.get(key) else { return };
            if !entry.creating || entry.session.is_some() {
                return;
            }
            state.entries.remove(key).map(|entry| entry.notify)
        };
        if let Some(notify) = notify {
            notify.notify_waiters();
        }
    }

    fn release(&self, key: &CompatibilityKey) {
        let session = {
            let mut state = self.state.lock();
            let Some(entry) = state.entries.get_mut(key) else { return };
            entry.members = entry.members.saturating_sub(1);
            if entry.members == 0 && !entry.creating {
                state.entries.remove(key).and_then(|entry| entry.session)
            } else {
                None
            }
        };
        drop(session);
    }

    #[cfg(test)]
    fn session_count(&self) -> usize {
        self.state.lock().entries.len()
    }
}

pub struct SharedStreamSessionLease {
    pub(crate) session: StreamSession,
    registry: SharedClientSessionRegistry,
    key: CompatibilityKey,
}

impl Drop for SharedStreamSessionLease {
    fn drop(&mut self) {
        self.registry.release(&self.key);
    }
}

pub struct SharedDatagramSessionLease {
    pub(crate) session: Arc<SharedDatagramSession>,
    registry: SharedClientSessionRegistry,
    key: CompatibilityKey,
}

impl Drop for SharedDatagramSessionLease {
    fn drop(&mut self) {
        self.registry.release(&self.key);
    }
}

fn compatibility_key(options: &SessionOptions, style: &str) -> CompatibilityKey {
    let mut safe_options = options.clone();
    let identity = match &safe_options.destination {
        DestinationKind::Transient => CompatibilityIdentity::Transient,
        DestinationKind::Persistent { private_key } => {
            CompatibilityIdentity::Persistent(private_key.clone())
        }
    };
    safe_options.nickname.clear();
    safe_options.destination = DestinationKind::Transient;
    CompatibilityKey {
        style: style.to_owned(),
        identity,
        session_options: format!("{safe_options:?}"),
        session_options_identity: additional_options_identity(&safe_options),
    }
}

/// Preserve exact custom-option equality for sharing without putting values in
/// the key's Debug/Display output. Yosemite's SessionOption Debug intentionally
/// redacts values, which is suitable for diagnostics but insufficient for
/// compatibility identity.
fn additional_options_identity(options: &SessionOptions) -> String {
    options
        .additional_options
        .iter()
        .map(|option| {
            format!(
                "{}:{}:{}:{};",
                option.key().len(),
                option.value().len(),
                option.key(),
                option.value()
            )
        })
        .collect()
}

/// Translate a client definition and its owned identity into Yosemite options.
pub(crate) async fn build_client_session_options(
    definition: &TunnelDefinition,
    sam_tcp_port: u16,
    store: Option<&ClientDestinationStore>,
) -> BackendResult<SessionOptions> {
    let identity = if definition.options.new_dest.is_some()
        || definition.options.persistent_client_key.is_some()
        || definition.options.priv_key_file.is_some()
    {
        let Some(store) = store else {
            return Err(BackendError::Internal {
                message: "client destination owner unavailable".to_string(),
            });
        };
        store.active(definition.name.as_str()).await.map_err(|_| BackendError::Internal {
            message: "client destination owner unavailable".to_string(),
        })?
    } else {
        None
    };
    let destination = identity.map_or(DestinationKind::Transient, |identity| {
        DestinationKind::Persistent {
            private_key: identity.as_str().to_owned(),
        }
    });
    build_session_options(definition, sam_tcp_port, false, destination)
}

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

    apply_session_wire_options(&mut options, &definition.options, definition.tunnel_type)?;

    Ok(options)
}

/// Apply validated generic Yosemite session-wire settings through the one
/// session builder used by every affected backend.
pub(crate) fn apply_session_wire_options(
    options: &mut SessionOptions,
    tunnel_options: &TunnelOptions,
    tunnel_type: TunnelType,
) -> BackendResult<()> {
    if let Some(value) = tunnel_options.tunnel_variance {
        options.inbound_len_variance = value as isize;
        options.outbound_len_variance = value as isize;
    }
    if let Some(value) = tunnel_options.tunnel_backup_quantity {
        options.inbound_backup_quantity = value as usize;
        options.outbound_backup_quantity = value as usize;
    }
    if let Some(value) = &tunnel_options.sig_type {
        options.signature_type = value.parse::<u16>().map_err(|_| BackendError::UnsupportedOption {
            tunnel_type,
            option: "SigType".to_owned(),
        })?;
    }
    for (key, value) in &tunnel_options.custom_options {
        options.add_session_option(key.clone(), value.clone()).map_err(|_| {
            BackendError::UnsupportedOption {
                tunnel_type,
                option: "CustomOptions".to_owned(),
            }
        })?;
    }
    Ok(())
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

/// Parse the residual client lifecycle fields before any runtime allocation.
/// `ConnectDelay` remains applied; `Close`, `CloseTime`, and `NewDest` are
/// M121-demoted to `blocked_primitive` and any supplied value fails here,
/// before listener/session allocation. Values remain in canonical raw config
/// for lossless round-trip, but no close/new-dest runtime effect is claimed.
pub(crate) fn client_lifecycle_config(
    definition: &TunnelDefinition,
) -> BackendResult<ClientLifecycleConfig> {
    let tunnel_type = definition.tunnel_type;
    if !tunnel_type.is_client() {
        for key in ["ConnectDelay", "Close", "CloseTime"] {
            if definition.raw_config.contains_key(key) {
                return Err(BackendError::UnsupportedOption {
                    tunnel_type,
                    option: key.to_owned(),
                });
            }
        }
        return Ok(ClientLifecycleConfig::DISABLED);
    }

    let connect_delay = raw_duration(
        definition,
        "ConnectDelay",
        0,
        MAX_CONNECT_DELAY,
    )?;
    // M121 §5.2 demotion: reference closeOnIdle observes I2P-session activity
    // (bytes/messages), not accepted local TCP handler count, and Yosemite
    // exposes no session-activity observation primitive. Fail closed.
    if definition.raw_config.contains_key("Close") {
        return Err(BackendError::UnsupportedOption {
            tunnel_type,
            option: "Close".to_owned(),
        });
    }
    if definition.raw_config.contains_key("CloseTime") {
        return Err(BackendError::UnsupportedOption {
            tunnel_type,
            option: "CloseTime".to_owned(),
        });
    }
    if definition.options.new_dest.is_some() {
        return Err(BackendError::UnsupportedOption {
            tunnel_type,
            option: "NewDest".to_owned(),
        });
    }

    Ok(ClientLifecycleConfig {
        connect_delay,
        close_on_idle: false,
        close_idle_time: DEFAULT_CLOSE_IDLE_TIME,
        new_dest_on_resume: false,
    })
}

fn raw_duration(
    definition: &TunnelDefinition,
    key: &str,
    minimum: u64,
    maximum: u64,
) -> BackendResult<Option<Duration>> {
    let Some(value) = definition.raw_config.get(key) else {
        return Ok(None);
    };
    let Some(value) = value.as_u64() else {
        return Err(BackendError::UnsupportedOption {
            tunnel_type: definition.tunnel_type,
            option: key.to_owned(),
        });
    };
    if !(minimum..=maximum).contains(&value) {
        return Err(BackendError::UnsupportedOption {
            tunnel_type: definition.tunnel_type,
            option: key.to_owned(),
        });
    }
    Ok(Some(Duration::from_millis(value)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i2pcontrol::domain::tunnel::{
        StartIntent, TunnelName, TunnelOptions, TunnelOwnership, TunnelRuntimeState,
    };
    use std::collections::BTreeMap;
    use tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        sync::oneshot,
    };

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
    fn m121_sigtype_seven_is_blocked_before_allocation_without_fallback() {
        // M121 Outcome C: even "7" fails as a configurable Proposal option.
        for value in ["7", "07", "1", "0", "11", "EdDSA_SHA512_Ed25519", ""] {
            let mut definition = definition();
            definition.options.sig_type = Some(value.to_owned());
            let error =
                build_session_options(&definition, 7656, false, DestinationKind::Transient)
                    .unwrap_err();
            assert!(
                matches!(error, BackendError::UnsupportedOption { option, .. } if option == "SigType"),
                "SigType {value:?} must fail before allocation"
            );
        }
        // Omitted SigType still builds with the Yosemite default (7) — that
        // default is router behavior, not Proposal SigType support.
        let definition = definition();
        let options =
            build_session_options(&definition, 7656, false, DestinationKind::Transient).unwrap();
        assert_eq!(options.signature_type, 7);
    }

    #[test]
    fn m121_close_closetime_newdest_fail_before_allocation_for_all_clients() {
        use crate::i2pcontrol::domain::tunnel::TunnelType::*;
        for tunnel_type in [
            Client,
            HttpClient,
            IrcClient,
            Socks,
            SocksIrc,
            ConnectClient,
        ] {
            let mut close_def = definition();
            close_def.tunnel_type = tunnel_type;
            close_def.raw_config.insert("Close".to_owned(), serde_json::json!(true));
            assert!(matches!(
                client_lifecycle_config(&close_def),
                Err(BackendError::UnsupportedOption { option, .. }) if option == "Close"
            ));

            let mut close_time_def = definition();
            close_time_def.tunnel_type = tunnel_type;
            close_time_def
                .raw_config
                .insert("CloseTime".to_owned(), serde_json::json!(1_000));
            assert!(matches!(
                client_lifecycle_config(&close_time_def),
                Err(BackendError::UnsupportedOption { option, .. }) if option == "CloseTime"
            ));

            let mut new_dest_def = definition();
            new_dest_def.tunnel_type = tunnel_type;
            new_dest_def.options.new_dest = Some(true);
            assert!(matches!(
                client_lifecycle_config(&new_dest_def),
                Err(BackendError::UnsupportedOption { option, .. }) if option == "NewDest"
            ));
            // validate_common_options is the earlier gate in production start
            // ordering (production.rs validates before stage/backend preflight).
            let error = crate::i2pcontrol::backends::options::validate_common_options(
                tunnel_type,
                &new_dest_def.options,
            )
            .unwrap_err();
            assert_eq!(error.to_string(), format!("{tunnel_type} does not support option NewDest"));
        }
    }

    #[test]
    fn supported_session_controls_translate_and_use_ssl_stays_blocked() {
        let mut definition = definition();
        definition.options.tunnel_variance = Some(1);
        definition.options.tunnel_backup_quantity = Some(1);
        let options =
            build_session_options(&definition, 7656, false, DestinationKind::Transient).unwrap();
        assert_eq!(options.inbound_len_variance, 1);
        assert_eq!(options.outbound_len_variance, 1);
        assert_eq!(options.inbound_backup_quantity, 1);
        assert_eq!(options.outbound_backup_quantity, 1);

        definition.options.use_ssl = Some(true);
        assert!(
            build_session_options(&definition, 7656, false, DestinationKind::Transient)
                .is_err()
        );
    }

    #[test]
    fn client_lifecycle_controls_are_bounded_and_fail_before_allocation() {
        // ConnectDelay remains applied; Close/CloseTime/NewDest are M121
        // demoted: any supplied value fails before allocation.
        let mut valid = definition();
        valid
            .raw_config
            .insert("ConnectDelay".to_owned(), serde_json::json!(60_000));
        let lifecycle = client_lifecycle_config(&valid).unwrap();
        assert_eq!(lifecycle.connect_delay, Some(Duration::from_secs(60)));
        assert!(!lifecycle.close_on_idle);
        assert!(!lifecycle.new_dest_on_resume);

        for (key, value) in [
            ("ConnectDelay", serde_json::json!(60_001)),
            ("ConnectDelay", serde_json::json!(-1)),
            ("Close", serde_json::json!(true)),
            ("Close", serde_json::json!(false)),
            ("Close", serde_json::json!("true")),
            ("CloseTime", serde_json::json!(0)),
            ("CloseTime", serde_json::json!(1)),
        ] {
            let mut invalid = definition();
            invalid.raw_config.insert(key.to_owned(), value);
            assert!(matches!(
                client_lifecycle_config(&invalid),
                Err(BackendError::UnsupportedOption { option, .. }) if option == key
            ));
        }

        let mut close_time_without_close = definition();
        close_time_without_close
            .raw_config
            .insert("CloseTime".to_owned(), serde_json::json!(1));
        assert!(matches!(
            client_lifecycle_config(&close_time_without_close),
            Err(BackendError::UnsupportedOption { option, .. }) if option == "CloseTime"
        ));

        let mut new_dest_without_close = definition();
        new_dest_without_close.options.new_dest = Some(true);
        assert!(matches!(
            client_lifecycle_config(&new_dest_without_close),
            Err(BackendError::UnsupportedOption { option, .. }) if option == "NewDest"
        ));

        // M121: NewDest=false is also blocked (no accept-inert). Close is
        // blocked even with Shared (fail on Close first).
        let mut new_dest_false = definition();
        new_dest_false.options.new_dest = Some(false);
        assert!(matches!(
            client_lifecycle_config(&new_dest_false),
            Err(BackendError::UnsupportedOption { option, .. }) if option == "NewDest"
        ));

        let mut shared_close = definition();
        shared_close
            .raw_config
            .insert("Close".to_owned(), serde_json::json!(true));
        shared_close.options.shared = Some(true);
        assert!(matches!(
            client_lifecycle_config(&shared_close),
            Err(BackendError::UnsupportedOption { option, .. }) if option == "Close"
        ));

        let mut persistent_new_dest = definition();
        persistent_new_dest.options.new_dest = Some(true);
        persistent_new_dest.options.persistent_client_key = Some(true);
        assert!(matches!(
            client_lifecycle_config(&persistent_new_dest),
            Err(BackendError::UnsupportedOption { option, .. }) if option == "NewDest"
        ));
    }

    #[test]
    fn generic_session_wire_adapter_maps_closed_yosemite_fields_without_fallback() {
        let mut definition = definition();
        definition.options.tunnel_variance = Some(-2);
        definition.options.tunnel_backup_quantity = Some(3);
        definition.options.sig_type = Some("11".to_owned());
        definition
            .options
            .custom_options
            .insert("i2cp.custom".to_owned(), "safe-value".to_owned());

        let mut options = SessionOptions::default();
        apply_session_wire_options(&mut options, &definition.options, definition.tunnel_type)
            .unwrap();

        assert_eq!(options.inbound_len_variance, -2);
        assert_eq!(options.outbound_len_variance, -2);
        assert_eq!(options.inbound_backup_quantity, 3);
        assert_eq!(options.outbound_backup_quantity, 3);
        assert_eq!(options.signature_type, 11);
        assert_eq!(options.additional_options[0].key(), "i2cp.custom");
    }

    #[test]
    fn generic_session_wire_adapter_rejects_invalid_signature_without_defaulting() {
        let mut definition = definition();
        definition.options.sig_type = Some("not-a-signature".to_owned());
        let mut options = SessionOptions::default();

        assert!(matches!(
            apply_session_wire_options(&mut options, &definition.options, definition.tunnel_type),
            Err(BackendError::UnsupportedOption { option, .. }) if option == "SigType"
        ));
        assert_eq!(options.signature_type, 7);
    }

    #[tokio::test]
    async fn generic_session_wire_adapter_reaches_fork_session_create_serializer() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let (command_tx, command_rx) = oneshot::channel();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (read_half, mut write_half) = stream.into_split();
            let mut reader = BufReader::new(read_half);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            assert_eq!(line, "HELLO VERSION\n");
            write_half
                .write_all(b"HELLO REPLY RESULT=OK VERSION=3.3\n")
                .await
                .unwrap();
            line.clear();
            reader.read_line(&mut line).await.unwrap();
            command_tx.send(line).unwrap();
            write_half
                .write_all(b"SESSION STATUS RESULT=OK DESTINATION=local\n")
                .await
                .unwrap();
        });

        let mut definition = definition();
        definition.options.tunnel_variance = Some(-2);
        definition.options.tunnel_backup_quantity = Some(3);
        definition.options.sig_type = Some("11".to_owned());
        definition
            .options
            .custom_options
            .insert("i2cp.custom".to_owned(), "safe-value".to_owned());
        let mut options = SessionOptions {
            samv3_tcp_port: port,
            nickname: "adapter-test".to_owned(),
            ..Default::default()
        };
        apply_session_wire_options(&mut options, &definition.options, definition.tunnel_type)
            .unwrap();

        let _session = Session::<style::Stream>::new(options).await.unwrap();
        let command = command_rx.await.unwrap();
        assert!(command.contains("inbound.lengthVariance=-2"));
        assert!(command.contains("inbound.backupQuantity=3"));
        assert!(command.contains("outbound.lengthVariance=-2"));
        assert!(command.contains("outbound.backupQuantity=3"));
        assert!(command.contains("SIGNATURE_TYPE=11"));
        assert!(command.contains("i2cp.custom=safe-value"));
        server.await.unwrap();
    }

    async fn fake_sam_session_create_command(mut options: SessionOptions) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        options.samv3_tcp_port = listener.local_addr().unwrap().port();
        let (command_tx, command_rx) = oneshot::channel();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (read_half, mut write_half) = stream.into_split();
            let mut reader = BufReader::new(read_half);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            assert_eq!(line, "HELLO VERSION\n");
            write_half
                .write_all(b"HELLO REPLY RESULT=OK VERSION=3.3\n")
                .await
                .unwrap();
            line.clear();
            reader.read_line(&mut line).await.unwrap();
            command_tx.send(line).unwrap();
            write_half
                .write_all(b"SESSION STATUS RESULT=OK DESTINATION=local\n")
                .await
                .unwrap();
        });

        let _session = Session::<style::Stream>::new(options).await.unwrap();
        let command = command_rx.await.unwrap();
        server.await.unwrap();
        command
    }

    /// M124: the corrected Y005 generic LeaseSet API is reachable from an
    /// I2PControl-only path without any Proposal mapping.
    ///
    /// This constructs validated Y004 session options directly (no
    /// `EncryptLeaseSet`/`LeaseSetClientAuths` Proposal translation, which
    /// remains owned by a future M113 successor) and observes the canonical
    /// wire at a local fake SAM endpoint. Fixture key material mirrors
    /// Yosemite Y004's own public test vectors; it is not router keying
    /// material and never leaves the test.
    #[tokio::test]
    async fn m124_y005_coherent_leaseset_wire_is_reachable_at_fake_sam() {
        use yosemite_i2pcontrol::{LeaseSetClientAuth, SessionOptions as YosemiteOptions};

        const LEASE_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        const PRIVATE_KEY: &str = "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=";
        const SIGNING_KEY: &str = "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC=";
        const SECRET: &str = "c2VjcmV0LXZhbHVlLWZpeHR1cmU=";

        let mut options = YosemiteOptions {
            nickname: "m124-leaseset-reachability".to_owned(),
            encrypt_lease_set: true,
            lease_set_auth_type: 1,
            lease_set_blinded_type: 10,
            lease_set_type: 5,
            lease_set_key: Some(LEASE_KEY.to_owned()),
            lease_set_private_key: Some(PRIVATE_KEY.to_owned()),
            lease_set_secret: Some(SECRET.to_owned()),
            lease_set_signing_private_key: Some(SIGNING_KEY.to_owned()),
            ..Default::default()
        };
        options
            .add_lease_set_client_auth(LeaseSetClientAuth::dh("alice", LEASE_KEY).unwrap())
            .unwrap();

        let command = fake_sam_session_create_command(options).await;

        // Representative corrected type-domain values.
        assert!(command.contains("i2cp.encryptLeaseSet=true"));
        assert!(command.contains("i2cp.leaseSetAuthType=1"));
        assert!(command.contains("i2cp.leaseSetBlindedType=10"));
        assert!(command.contains("i2cp.leaseSetType=5"));
        assert!(command.contains(&format!("i2cp.leaseSetKey={LEASE_KEY}")));
        assert!(command.contains(&format!("i2cp.leaseSetSecret={SECRET}")));
        // Canonical private/signing spellings (Y003 emitted truncated aliases).
        assert!(command.contains(&format!("i2cp.leaseSetPrivateKey={PRIVATE_KEY}")));
        assert!(command.contains(&format!("i2cp.leaseSetSigningPrivateKey={SIGNING_KEY}")));
        assert!(!command.contains("i2cp.leaseSetPrivKey="));
        assert!(!command.contains("i2cp.leaseSetSigningPrivKey="));
        // Mode-aware client-auth namespaces with deterministic per-mode numbering.
        assert_eq!(command.matches("i2cp.leaseSetClient.dh.").count(), 1);
        assert!(command.contains(&format!("i2cp.leaseSetClient.dh.0=YWxpY2U=:{LEASE_KEY}")));
        assert!(!command.contains("i2cp.leaseSetClient.psk."));
        assert!(!command.contains("i2cp.leaseSetClientAuth"));

        let mut psk_options = YosemiteOptions {
            nickname: "m124-psk-leaseset".to_owned(),
            lease_set_auth_type: 2,
            lease_set_type: 5,
            ..Default::default()
        };
        psk_options
            .add_lease_set_client_auth(LeaseSetClientAuth::psk("bob", LEASE_KEY).unwrap())
            .unwrap();
        let psk_command = fake_sam_session_create_command(psk_options).await;
        assert!(psk_command.contains("i2cp.leaseSetAuthType=2"));
        assert!(psk_command.contains("i2cp.leaseSetType=5"));
        assert!(psk_command.contains(&format!("i2cp.leaseSetClient.psk.0=Ym9i:{LEASE_KEY}")));
        assert!(!psk_command.contains("i2cp.leaseSetClient.dh."));
    }

    /// M124: malformed corrected-API values reject before SAM wire bytes.
    ///
    /// Constructor/bounded-collection errors never reach the network, and
    /// numeric-domain violations fail inside `SessionController::new` before
    /// the TCP connect (proved by asserting `InvalidOption` against a closed
    /// port, where a post-connect failure would surface as I/O instead).
    #[tokio::test]
    async fn m124_y005_malformed_leaseset_values_reject_before_wire() {
        use yosemite_i2pcontrol::{
            Error as YosemiteError, LeaseSetClientAuth, ProtocolError as YosemiteProtocolError,
        };

        const KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

        // Opaque preformatted fragments, empty names, control bytes, and
        // non-I2P-base64 material never become entries.
        assert!(LeaseSetClientAuth::dh("alice", "not-base64!!").is_err());
        assert!(LeaseSetClientAuth::dh("", KEY).is_err());
        assert!(LeaseSetClientAuth::dh("bad\nname", KEY).is_err());
        // Unused-bit violations reject even when the alphabet/padding shape is right.
        assert!(
            LeaseSetClientAuth::psk("alice", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB=")
                .is_err()
        );
        assert!(LeaseSetClientAuth::psk("alice", "short").is_err());

        // Duplicates within one mode reject; the same name may appear once
        // per distinct reference namespace.
        let mut options = SessionOptions::default();
        options
            .add_lease_set_client_auth(LeaseSetClientAuth::dh("alice", KEY).unwrap())
            .unwrap();
        assert!(options
            .add_lease_set_client_auth(LeaseSetClientAuth::dh("alice", KEY).unwrap())
            .is_err());
        options
            .add_lease_set_client_auth(LeaseSetClientAuth::psk("alice", KEY).unwrap())
            .unwrap();

        // Canonical typed keys and numbered client-auth namespaces are
        // reserved on the generic path.
        let mut generic = SessionOptions::default();
        assert!(generic.add_session_option("i2cp.leaseSetPrivateKey", "x").is_err());
        assert!(generic.add_session_option("i2cp.leaseSetSigningPrivateKey", "x").is_err());
        assert!(generic.add_session_option("i2cp.leaseSetClient.dh.0", "x").is_err());
        assert!(generic.add_session_option("i2cp.leaseSetClient.psk.0", "x").is_err());
        assert!(generic.add_session_option("i2cp.leaseSetClientAuth.0", "x").is_err());

        // Numeric-domain violations fail before the TCP connect. The closed
        // port guarantees a post-validation failure would be I/O, not
        // `InvalidOption`.
        for mut invalid in [
            SessionOptions {
                lease_set_auth_type: 3,
                ..Default::default()
            },
            SessionOptions {
                lease_set_blinded_type: 65_536,
                ..Default::default()
            },
            SessionOptions {
                lease_set_type: 0,
                ..Default::default()
            },
            SessionOptions {
                lease_set_key: Some("not base64!!".to_owned()),
                ..Default::default()
            },
            SessionOptions {
                lease_set_auth_type: 1,
                lease_set_type: 5,
                lease_set_client_auths: vec![LeaseSetClientAuth::psk("alice", KEY).unwrap()],
                ..Default::default()
            },
            SessionOptions {
                lease_set_auth_type: 1,
                lease_set_type: 5,
                lease_set_client_auths: vec![
                    LeaseSetClientAuth::dh("alice", KEY).unwrap(),
                    LeaseSetClientAuth::psk("bob", KEY).unwrap(),
                ],
                ..Default::default()
            },
            SessionOptions {
                lease_set_auth_type: 0,
                lease_set_type: 5,
                lease_set_client_auths: vec![LeaseSetClientAuth::dh("alice", KEY).unwrap()],
                ..Default::default()
            },
            SessionOptions {
                lease_set_auth_type: 1,
                lease_set_type: 3,
                ..Default::default()
            },
        ] {
            invalid.samv3_tcp_port = 1;
            invalid.nickname = "m124-invalid-leaseset".to_owned();
            assert!(
                matches!(
                    Session::<style::Stream>::new(invalid).await,
                    Err(YosemiteError::Protocol(
                        YosemiteProtocolError::InvalidOption
                    ))
                ),
                "invalid LeaseSet options must fail before wire"
            );
        }
    }

    /// M122: LeaseSet fixture material never enters debug/diagnostics.
    #[test]
    fn m122_y004_leaseset_material_is_redacted_from_debug() {
        use yosemite_i2pcontrol::LeaseSetClientAuth;

        const KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        const PRIVATE_KEY: &str = "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=";
        const SIGNING_KEY: &str = "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC=";

        let mut options = SessionOptions {
            lease_set_key: Some(KEY.to_owned()),
            lease_set_private_key: Some(PRIVATE_KEY.to_owned()),
            lease_set_secret: Some("c2VjcmV0LXZhbHVlLWZpeHR1cmU=".to_owned()),
            lease_set_signing_private_key: Some(SIGNING_KEY.to_owned()),
            ..Default::default()
        };
        options
            .add_lease_set_client_auth(LeaseSetClientAuth::dh("alice", KEY).unwrap())
            .unwrap();
        let debug = format!("{options:?}");
        for secret in [
            KEY,
            PRIVATE_KEY,
            SIGNING_KEY,
            "c2VjcmV0LXZhbHVlLWZpeHR1cmU=",
        ] {
            assert!(!debug.contains(secret), "debug leaked LeaseSet material");
        }
        let auth_debug = format!("{:?}", LeaseSetClientAuth::dh("alice", KEY).unwrap());
        assert!(
            !auth_debug.contains(KEY),
            "client-auth debug leaked key material"
        );
        assert!(auth_debug.contains("<redacted>"));
    }

    #[test]
    fn compatibility_key_includes_session_settings_and_identity_without_exposing_it() {
        let first = SessionOptions {
            inbound_len: 2,
            destination: DestinationKind::Persistent {
                private_key: "c2VjcmV0LWtleQ==".to_owned(),
            },
            ..Default::default()
        };
        let mut same = first.clone();
        same.nickname = "different-name".to_owned();
        let mut different = first.clone();
        different.outbound_len = 4;
        let mut different_wire = first.clone();
        different_wire.signature_type = 7;
        different_wire.inbound_len_variance = -2;
        different_wire.outbound_backup_quantity = 3;
        different_wire
            .add_session_option("i2cp.custom".to_owned(), "safe-value".to_owned())
            .unwrap();
        let mut different_custom_value = different_wire.clone();
        different_custom_value.additional_options[0] =
            yosemite_i2pcontrol::SessionOption::new("i2cp.custom", "different-value").unwrap();
        let mut different_identity = first.clone();
        different_identity.destination = DestinationKind::Persistent {
            private_key: "b3RoZXIta2V5".to_owned(),
        };
        let first_key = compatibility_key(&first, "stream");
        assert_eq!(first_key, compatibility_key(&same, "stream"));
        assert_ne!(first_key, compatibility_key(&different, "stream"));
        assert_ne!(first_key, compatibility_key(&different_wire, "stream"));
        assert_ne!(
            compatibility_key(&different_wire, "stream"),
            compatibility_key(&different_custom_value, "stream")
        );
        assert_ne!(first_key, compatibility_key(&different_identity, "stream"));
        let debug = format!("{first_key:?}");
        let display = format!("{first_key}");
        assert!(!debug.contains("c2VjcmV0"));
        assert!(!display.contains("c2VjcmV0"));
        assert!(debug.contains("persistent"));
    }

    #[tokio::test]
    async fn waiter_registration_precedes_notification() {
        let notify = Arc::new(Notify::new());
        let mut waiter = Box::pin(Arc::clone(&notify).notified_owned());
        waiter.as_mut().enable();
        notify.notify_waiters();
        tokio::time::timeout(std::time::Duration::from_secs(1), waiter)
            .await
            .expect("enabled waiter must observe notify_waiters");
    }

    #[test]
    fn dropped_creator_reservation_reopens_its_key() {
        let registry = SharedClientSessionRegistry::new();
        let key = compatibility_key(&SessionOptions::default(), "stream");
        registry.state.lock().entries.insert(
            key.clone(),
            SharedSessionEntry {
                session: None,
                members: 0,
                creating: true,
                notify: Arc::new(Notify::new()),
            },
        );
        drop(CreationReservation::new(registry.clone(), key));
        assert_eq!(registry.session_count(), 0);
    }
}
