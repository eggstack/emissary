//! Common Proposal 170 session option translation.
//!
//! This module is the single I2PControl-owned boundary between canonical
//! tunnel options and Yosemite's `SessionOptions`. It intentionally exposes
//! no Proposal 170 types to the router or core crates.

use std::{
    collections::BTreeMap,
    hash::{Hash, Hasher},
    sync::Arc,
};

use parking_lot::Mutex;
use tokio::sync::{broadcast, mpsc, oneshot, Notify};
use yosemite::{style, DatagramOptions, DestinationKind, Session, SessionOptions};

use super::super::{options::validate_common_options, BackendError, BackendResult};
use crate::i2pcontrol::client_secret_store::ClientDestinationStore;
use crate::i2pcontrol::domain::tunnel::{TunnelDefinition, TunnelType};

type StreamSession = Arc<Mutex<Session<style::Stream>>>;

const DATAGRAM_BUFFER_SIZE: usize = 4095;

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
    entries: BTreeMap<String, SharedSessionEntry>,
}

/// Bounded owner for I2PControl-shared client sessions.
///
/// The compatibility key contains every translated Yosemite session setting
/// and a non-reversible fingerprint of persistent identity material. Session
/// construction happens outside the bookkeeping lock; the per-key creator
/// reservation prevents concurrent equivalent starts from duplicating owners.
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
            let (notify, create) = {
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
                    (Arc::clone(&entry.notify), !entry.creating)
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
                    (notify, true)
                }
            };
            if !create {
                notify.notified().await;
                continue;
            }
            let result = Session::<style::Stream>::new(options.clone())
                .await
                .map(|session| Arc::new(Mutex::new(session)))
                .map_err(|_| "client shared session setup failed".to_string());
            let mut state = self.state.lock();
            let entry = state.entries.get_mut(&key).expect("creator reservation exists");
            entry.creating = false;
            match result {
                Ok(session) => {
                    entry.members = 1;
                    entry.session = Some(SessionHandle::Stream(Arc::clone(&session)));
                    entry.notify.notify_waiters();
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
            let (notify, create) = {
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
                    (Arc::clone(&entry.notify), !entry.creating)
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
                    (notify, true)
                }
            };
            if !create {
                notify.notified().await;
                continue;
            }
            let result = Session::<style::Repliable>::new(options.clone())
                .await
                .map(SharedDatagramSession::spawn)
                .map_err(|_| "client shared datagram session setup failed".to_string());
            let mut state = self.state.lock();
            let entry = state.entries.get_mut(&key).expect("creator reservation exists");
            entry.creating = false;
            match result {
                Ok(session) => {
                    entry.members = 1;
                    entry.session = Some(SessionHandle::Datagram(Arc::clone(&session)));
                    entry.notify.notify_waiters();
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
                    return Err(error);
                }
            }
        }
    }

    fn release(&self, key: &str) {
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
    key: String,
}

impl Drop for SharedStreamSessionLease {
    fn drop(&mut self) {
        self.registry.release(&self.key);
    }
}

pub struct SharedDatagramSessionLease {
    pub(crate) session: Arc<SharedDatagramSession>,
    registry: SharedClientSessionRegistry,
    key: String,
}

impl Drop for SharedDatagramSessionLease {
    fn drop(&mut self) {
        self.registry.release(&self.key);
    }
}

fn compatibility_key(options: &SessionOptions, style: &str) -> String {
    let mut safe_options = options.clone();
    let identity = match &safe_options.destination {
        DestinationKind::Transient => "transient".to_string(),
        DestinationKind::Persistent { private_key } => {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            private_key.hash(&mut hasher);
            format!("persistent:{:016x}", hasher.finish())
        }
    };
    safe_options.nickname.clear();
    safe_options.destination = DestinationKind::Transient;
    format!("{style}|{identity}|{safe_options:?}")
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
        ] {
            let mut definition = definition();
            set(&mut definition.options);
            assert!(
                build_session_options(&definition, 7656, false, DestinationKind::Transient)
                    .is_err()
            );
        }
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
        let mut different_identity = first.clone();
        different_identity.destination = DestinationKind::Persistent {
            private_key: "b3RoZXIta2V5".to_owned(),
        };
        let first_key = compatibility_key(&first, "stream");
        assert_eq!(first_key, compatibility_key(&same, "stream"));
        assert_ne!(first_key, compatibility_key(&different, "stream"));
        assert_ne!(first_key, compatibility_key(&different_identity, "stream"));
        assert!(!first_key.contains("c2VjcmV0"));
    }
}
