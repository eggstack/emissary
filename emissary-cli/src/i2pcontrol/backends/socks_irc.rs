//! SOCKS-IRC composition for Proposal 170.
//!
//! Negotiation and target routing are shared with `socks`; only the payload
//! handler changes, and it always enters M066's stateful IRC relay filter.

use std::{fmt, sync::Arc};

use super::{
    options::SOCKS_IRC_OPTIONS,
    socks::{config_for, PayloadMode, SocksRuntimeSupervisor},
    BackendError, BackendResult, BackendStatus, TunnelBackend,
};
use crate::i2pcontrol::{
    address_book_runtime::RuntimeAddressBookHandle,
    domain::tunnel::{TunnelDefinition, TunnelOwnership, TunnelType},
};

/// Real filtered backend for Proposal 170 `socksirc`.
#[derive(Clone)]
pub struct SocksIrcTunnelBackend {
    supervisor: SocksRuntimeSupervisor,
    sam_tcp_port: u16,
    address_book: Option<Arc<RuntimeAddressBookHandle>>,
}

impl fmt::Debug for SocksIrcTunnelBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SocksIrcTunnelBackend")
            .field("sam_tcp_port", &self.sam_tcp_port)
            .finish_non_exhaustive()
    }
}

impl SocksIrcTunnelBackend {
    pub fn new(sam_tcp_port: u16) -> Self {
        Self {
            supervisor: SocksRuntimeSupervisor::new(TunnelType::SocksIrc),
            sam_tcp_port,
            address_book: None,
        }
    }

    pub fn with_address_book(mut self, address_book: Arc<RuntimeAddressBookHandle>) -> Self {
        self.address_book = Some(address_book);
        self
    }
}

#[async_trait::async_trait]
impl TunnelBackend for SocksIrcTunnelBackend {
    fn tunnel_type(&self) -> TunnelType {
        TunnelType::SocksIrc
    }

    async fn start(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        // Keep this check local to the composing backend so a future option
        // matrix change cannot accidentally make SOCKS-IRC broader than M066.
        if definition.ownership != TunnelOwnership::ControlPlane {
            return Err(BackendError::InvalidState {
                tunnel_type: TunnelType::SocksIrc,
                current_state: definition.runtime_state,
                attempted_action: "start",
            });
        }
        let mut config = config_for(
            definition,
            TunnelType::SocksIrc,
            self.sam_tcp_port,
            self.address_book.clone(),
            SOCKS_IRC_OPTIONS,
        )?;
        config.session_options = super::runtime::session::build_session_options(
            definition,
            self.sam_tcp_port,
            false,
            yosemite::DestinationKind::Transient,
        )?;
        self.supervisor.start(config, PayloadMode::Irc).await
    }

    async fn stop(&self, definition: &TunnelDefinition) -> BackendResult<()> {
        self.supervisor.stop(definition.name.as_str()).await
    }

    fn inspect(&self, definition: &TunnelDefinition) -> BackendStatus {
        let (runtime_state, message) = self.supervisor.inspect(definition.name.as_str());
        BackendStatus {
            tunnel_type: TunnelType::SocksIrc,
            runtime_state,
            message: message.to_owned(),
            destination: None,
        }
    }
}
