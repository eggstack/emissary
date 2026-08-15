//! Small, bounded runtime owners for future Proposal 170 tunnel backends.
//!
//! These helpers own listeners, Yosemite sessions, and connection tasks. They
//! deliberately do not contain HTTP, IRC, SOCKS, or Streamr policy.

#![allow(dead_code, unused_imports)]

mod accepted_server;
mod admission;
mod client_listener;
mod task_group;

pub use admission::{
    AdmissionDecision, AdmissionLease, AdmissionRejection, ServerAdmissionPolicy,
    ServerAdmissionState,
};

pub use accepted_server::{
    run_accepted_server, AcceptedServerConnection, AcceptedServerHandler,
    AcceptedServerRuntimeConfig, AcceptedServerRuntimeError, TrustedPeerIdentity,
};
pub use client_listener::{
    run_client_listener, ClientConnectionHandler, ClientListenerRuntimeConfig,
    ClientListenerRuntimeError, ClientStreamConnector,
};
