//! Test-only re-export module for `peer_identity` fixtures used by other
//! I2PControl backend test suites (notably the generic-server raw-relay
//! fixture). Production code paths continue to consume
//! [`crate::i2pcontrol::backends::runtime::TrustedPeerIdentity`].

pub use super::peer_identity_impl::*;
