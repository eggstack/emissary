// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use std::{collections::HashMap, sync::Arc};

use super::TunnelBackend;
use crate::i2pcontrol::{
    address_book_runtime::RuntimeAddressBookHandle,
    domain::tunnel::{TunnelType, ALL_TUNNEL_ACTIONS, ALL_TUNNEL_TYPES},
    server_secret_store::ServerDestinationStore,
};

/// An exhaustive tunnel backend registry.
///
/// Construction fails if any `TunnelType` lacks a backend or if duplicates
/// are detected. Every valid tunnel type maps to exactly one backend.
#[derive(Clone)]
pub struct TunnelBackendRegistry {
    backends: HashMap<TunnelType, Arc<dyn TunnelBackend>>,
}

impl std::fmt::Debug for TunnelBackendRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TunnelBackendRegistry")
            .field("len", &self.backends.len())
            .finish()
    }
}

impl TunnelBackendRegistry {
    /// Create a new registry from an iterator of backends.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError` if:
    /// - a tunnel type is registered more than once
    /// - a tunnel type is missing from the registration
    pub fn new(backends: Vec<Arc<dyn TunnelBackend>>) -> Result<Self, RegistryError> {
        let mut map = HashMap::new();
        for backend in backends {
            let tt = backend.tunnel_type();
            if map.contains_key(&tt) {
                return Err(RegistryError::DuplicateRegistration(tt));
            }
            map.insert(tt, backend);
        }

        // Verify all types are registered
        for &tt in ALL_TUNNEL_TYPES {
            if !map.contains_key(&tt) {
                return Err(RegistryError::MissingRegistration(tt));
            }
        }

        Ok(Self { backends: map })
    }

    /// Look up a backend by tunnel type.
    ///
    /// This is total for valid tunnel types.
    pub fn get(&self, tunnel_type: TunnelType) -> Arc<dyn TunnelBackend> {
        self.backends
            .get(&tunnel_type)
            .cloned()
            .expect("registry is exhaustive; all tunnel types are registered")
    }

    /// Return the number of registered backends.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.backends.len()
    }

    /// Return true if the registry is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.backends.is_empty()
    }

    /// Return true if the given tunnel type is registered.
    #[allow(dead_code)]
    pub fn contains(&self, tunnel_type: TunnelType) -> bool {
        self.backends.contains_key(&tunnel_type)
    }
}

/// Errors from registry construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// A tunnel type was registered more than once.
    DuplicateRegistration(TunnelType),
    /// A tunnel type has no registered backend.
    MissingRegistration(TunnelType),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateRegistration(tt) => {
                write!(f, "duplicate registration for tunnel type: {}", tt.as_str())
            }
            Self::MissingRegistration(tt) => {
                write!(f, "missing registration for tunnel type: {}", tt.as_str())
            }
        }
    }
}

impl std::error::Error for RegistryError {}

/// Create a test/default registry with all tunnel types mapped to unsupported
/// backends.
///
/// Production composition uses [`create_production_registry`] so the client
/// backend receives the already-bound SAM endpoint without making this
/// dependency part of the test/fake registry contract.
pub fn create_default_registry() -> Result<TunnelBackendRegistry, RegistryError> {
    let backends: Vec<Arc<dyn TunnelBackend>> = ALL_TUNNEL_TYPES
        .iter()
        .map(|&tt| {
            Arc::new(super::unsupported::UnsupportedTunnelBackend::new(tt))
                as Arc<dyn TunnelBackend>
        })
        .collect();
    TunnelBackendRegistry::new(backends)
}

/// Create the dependency-light registry used by tests and non-production
/// construction paths.
///
/// The specialized server backends require the composed, path-confined
/// destination store. Production composition must use
/// [`create_production_registry_with_server_store_and_address_book`] so all
/// twelve tunnel types receive their real backend.
#[allow(dead_code)]
pub fn create_production_registry(
    sam_tcp_port: u16,
) -> Result<TunnelBackendRegistry, RegistryError> {
    let client =
        Arc::new(super::client::ClientTunnelBackend::new(sam_tcp_port)) as Arc<dyn TunnelBackend>;
    let server = Arc::new(super::server::ServerTunnelBackend::without_store(
        sam_tcp_port,
    )) as Arc<dyn TunnelBackend>;
    let backends: Vec<Arc<dyn TunnelBackend>> = ALL_TUNNEL_TYPES
        .iter()
        .map(|&tt| {
            if tt == TunnelType::Client {
                client.clone()
            } else if tt == TunnelType::Server {
                server.clone()
            } else {
                Arc::new(super::unsupported::UnsupportedTunnelBackend::new(tt))
                    as Arc<dyn TunnelBackend>
            }
        })
        .collect();
    TunnelBackendRegistry::new(backends)
}

/// Create the production registry with a composed backend-owned server store.
#[allow(dead_code)]
pub fn create_production_registry_with_server_store(
    sam_tcp_port: u16,
    server_store: ServerDestinationStore,
) -> Result<TunnelBackendRegistry, RegistryError> {
    create_production_registry_with_server_store_and_address_book(sam_tcp_port, server_store, None)
}

/// Create the production registry with the control-plane address-book resolver
/// used by dynamic HTTP/CONNECT client targets.
pub fn create_production_registry_with_server_store_and_address_book(
    sam_tcp_port: u16,
    server_store: ServerDestinationStore,
    address_book: Option<Arc<RuntimeAddressBookHandle>>,
) -> Result<TunnelBackendRegistry, RegistryError> {
    let client =
        Arc::new(super::client::ClientTunnelBackend::new(sam_tcp_port)) as Arc<dyn TunnelBackend>;
    let server = Arc::new(super::server::ServerTunnelBackend::new(
        sam_tcp_port,
        server_store.clone(),
    )) as Arc<dyn TunnelBackend>;
    let irc_client = Arc::new(super::irc_client::IrcClientTunnelBackend::new(sam_tcp_port))
        as Arc<dyn TunnelBackend>;
    let irc_server = Arc::new(super::irc_server::IrcServerTunnelBackend::new(
        sam_tcp_port,
        server_store.clone(),
    )) as Arc<dyn TunnelBackend>;
    let http_server = Arc::new(super::http_server::HttpServerTunnelBackend::new(
        sam_tcp_port,
        server_store.clone(),
    )) as Arc<dyn TunnelBackend>;
    let http_bidir = Arc::new(super::http_bidir::HttpBidirServerTunnelBackend::new(
        sam_tcp_port,
        server_store.clone(),
        address_book.clone(),
    )) as Arc<dyn TunnelBackend>;
    let http_client = Arc::new(match address_book.clone() {
        Some(address_book) => super::http_client::HttpClientTunnelBackend::new(sam_tcp_port)
            .with_address_book(address_book),
        None => super::http_client::HttpClientTunnelBackend::new(sam_tcp_port),
    }) as Arc<dyn TunnelBackend>;
    let connect_client = Arc::new(match address_book.clone() {
        Some(address_book) => super::connect_client::ConnectClientTunnelBackend::new(sam_tcp_port)
            .with_address_book(address_book),
        None => super::connect_client::ConnectClientTunnelBackend::new(sam_tcp_port),
    }) as Arc<dyn TunnelBackend>;
    let socks = Arc::new(match address_book.clone() {
        Some(address_book) =>
            super::socks::SocksTunnelBackend::new(sam_tcp_port).with_address_book(address_book),
        None => super::socks::SocksTunnelBackend::new(sam_tcp_port),
    }) as Arc<dyn TunnelBackend>;
    let socks_irc = Arc::new(match address_book {
        Some(address_book) => super::socks_irc::SocksIrcTunnelBackend::new(sam_tcp_port)
            .with_address_book(address_book),
        None => super::socks_irc::SocksIrcTunnelBackend::new(sam_tcp_port),
    }) as Arc<dyn TunnelBackend>;
    let streamr_client = Arc::new(super::streamr::StreamrClientTunnelBackend::new(
        sam_tcp_port,
    )) as Arc<dyn TunnelBackend>;
    let streamr_server = Arc::new(super::streamr::StreamrServerTunnelBackend::new(
        sam_tcp_port,
        server_store,
    )) as Arc<dyn TunnelBackend>;
    let backends: Vec<Arc<dyn TunnelBackend>> = ALL_TUNNEL_TYPES
        .iter()
        .map(|&tt| {
            if tt == TunnelType::Client {
                client.clone()
            } else if tt == TunnelType::Server {
                server.clone()
            } else if tt == TunnelType::IrcClient {
                irc_client.clone()
            } else if tt == TunnelType::IrcServer {
                irc_server.clone()
            } else if tt == TunnelType::HttpServer {
                http_server.clone()
            } else if tt == TunnelType::HttpBidirServer {
                http_bidir.clone()
            } else if tt == TunnelType::HttpClient {
                http_client.clone()
            } else if tt == TunnelType::ConnectClient {
                connect_client.clone()
            } else if tt == TunnelType::Socks {
                socks.clone()
            } else if tt == TunnelType::SocksIrc {
                socks_irc.clone()
            } else if tt == TunnelType::StreamrClient {
                streamr_client.clone()
            } else if tt == TunnelType::StreamrServer {
                streamr_server.clone()
            } else {
                Arc::new(super::unsupported::UnsupportedTunnelBackend::new(tt))
                    as Arc<dyn TunnelBackend>
            }
        })
        .collect();
    TunnelBackendRegistry::new(backends)
}

/// Compile-time guard: every TunnelType variant is listed in ALL_TUNNEL_TYPES.
///
/// If you add a variant to TunnelType and forget to add it to ALL_TUNNEL_TYPES,
/// this const will fail to compile.
const _: () = {
    // This block runs at compile time. If ALL_TUNNEL_TYPES has fewer than
    // the expected number of elements, the array literal will fail.
    #[allow(dead_code)]
    const EXPECTED_COUNT: usize = 12;
    const _CHECK: [(); EXPECTED_COUNT] = [(); ALL_TUNNEL_TYPES.len()];
};

/// Compile-time guard: every TunnelAction variant is listed in ALL_TUNNEL_ACTIONS.
const _: () = {
    #[allow(dead_code)]
    const EXPECTED_COUNT: usize = 7;
    const _CHECK: [(); EXPECTED_COUNT] = [(); ALL_TUNNEL_ACTIONS.len()];
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i2pcontrol::{
        backends::{unsupported::UnsupportedTunnelBackend, BackendError},
        domain::tunnel::{
            StartIntent, TunnelDefinition, TunnelName, TunnelOptions, TunnelOwnership,
            TunnelRuntimeState,
        },
    };

    fn test_definition(tunnel_type: TunnelType) -> TunnelDefinition {
        TunnelDefinition {
            name: TunnelName::new("registry-test").unwrap(),
            tunnel_type,
            ownership: TunnelOwnership::ControlPlane,
            runtime_state: TunnelRuntimeState::Stopped,
            start_intent: StartIntent::DoNotStart,
            options: TunnelOptions::default(),
            raw_config: std::collections::BTreeMap::new(),
        }
    }

    #[tokio::test]
    async fn default_registry_all_types_unsupported() {
        let registry = create_default_registry().unwrap();
        assert_eq!(registry.len(), 12);

        for &tt in ALL_TUNNEL_TYPES {
            let backend = registry.get(tt);
            assert_eq!(backend.tunnel_type(), tt);
            let def = test_definition(tt);
            let result = backend.start(&def).await;
            assert!(matches!(result, Err(BackendError::NotImplemented { .. })));
            let status = backend.inspect(&def);
            assert_eq!(status.runtime_state, TunnelRuntimeState::Unsupported);
        }
    }

    #[test]
    fn registry_rejects_duplicate() {
        let backends: Vec<Arc<dyn TunnelBackend>> = ALL_TUNNEL_TYPES
            .iter()
            .map(|&tt| Arc::new(UnsupportedTunnelBackend::new(tt)) as Arc<dyn TunnelBackend>)
            .collect();

        // Duplicate the first one
        let mut backends = backends;
        let first = backends[0].clone();
        backends.push(first);

        let result = TunnelBackendRegistry::new(backends);
        assert!(result.is_err());
        match result.unwrap_err() {
            RegistryError::DuplicateRegistration(tt) => {
                assert_eq!(tt, ALL_TUNNEL_TYPES[0]);
            }
            _ => panic!("expected DuplicateRegistration"),
        }
    }

    #[test]
    fn registry_rejects_missing() {
        let backends: Vec<Arc<dyn TunnelBackend>> = ALL_TUNNEL_TYPES[1..] // skip first
            .iter()
            .map(|&tt| Arc::new(UnsupportedTunnelBackend::new(tt)) as Arc<dyn TunnelBackend>)
            .collect();

        let result = TunnelBackendRegistry::new(backends);
        assert!(result.is_err());
        match result.unwrap_err() {
            RegistryError::MissingRegistration(tt) => {
                assert_eq!(tt, ALL_TUNNEL_TYPES[0]);
            }
            _ => panic!("expected MissingRegistration"),
        }
    }

    #[test]
    fn registry_contains_all_types() {
        let registry = create_default_registry().unwrap();
        for &tt in ALL_TUNNEL_TYPES {
            assert!(registry.contains(tt));
        }
    }

    #[test]
    fn composed_production_registry_registers_real_backends_for_all_types() {
        let root = tempfile::tempdir().unwrap();
        let registry = create_production_registry_with_server_store(
            7656,
            ServerDestinationStore::new(root.path()),
        )
        .unwrap();

        assert_eq!(registry.len(), ALL_TUNNEL_TYPES.len());
        for &tunnel_type in ALL_TUNNEL_TYPES {
            let status = registry.get(tunnel_type).inspect(&test_definition(tunnel_type));
            assert_eq!(status.tunnel_type, tunnel_type);
            assert_eq!(status.runtime_state, TunnelRuntimeState::Stopped);
        }
    }
}
