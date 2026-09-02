//! Small, bounded runtime owners for future Proposal 170 tunnel backends.
//!
//! These helpers own listeners, Yosemite sessions, and connection tasks. They
//! deliberately do not contain HTTP, IRC, SOCKS, or Streamr policy.

#![allow(dead_code, unused_imports)]

mod accepted_server;
mod access;
mod admission;
mod client_listener;
#[cfg(test)]
pub mod peer_identity;
mod peer_identity_impl;
pub mod session;
mod task_group;

pub use access::{AccessOption, ServerAccessPolicy};
pub use admission::{
    AdmissionDecision, AdmissionLease, AdmissionRejection, ServerAdmissionPolicy,
    ServerAdmissionState,
};

pub use accepted_server::{
    run_accepted_server, AcceptedServerConnection, AcceptedServerHandler,
    AcceptedServerRuntimeConfig, AcceptedServerRuntimeError,
};
pub use client_listener::{
    run_client_listener, run_generic_client, run_generic_client_with_shared_session,
    run_client_listener_with_shared_session, ClientConnectionHandler,
    ClientListenerRuntimeConfig, ClientListenerRuntimeError, ClientStreamConnector,
};
pub(crate) use session::{client_lifecycle_config, ClientLifecycleConfig};
pub use peer_identity_impl::{TrustedPeerIdentity, MAX_TRUSTED_DESTINATION_B64_TEXT};
