// Emissary CLI library root.
//
// This crate is primarily a binary, but this lib.rs re-exports modules
// needed for integration testing of the i2pcontrol subsystem.

#[cfg(feature = "i2pcontrol")]
pub mod config {
    /// I2CP options consumed by the startup client manager.
    #[derive(Debug, Clone, Default)]
    pub struct I2cpOptions {
        /// Optional lease-set encryption type.
        pub lease_set_enc_type: Option<String>,
    }

    /// Startup generic client tunnel configuration.
    #[derive(Debug, Clone)]
    pub struct ClientTunnelConfig {
        /// Tunnel name.
        pub name: String,
        /// Local bind interface.
        pub address: Option<String>,
        /// Local bind port.
        pub port: u16,
        /// Remote destination.
        pub destination: String,
        /// Remote destination port.
        pub destination_port: Option<u16>,
    }

    /// Startup generic client options.
    #[derive(Debug, Clone, Default)]
    pub struct ClientTunnelOptions {
        /// Optional I2CP options.
        pub i2cp: Option<I2cpOptions>,
    }

    /// Startup generic server tunnel configuration.
    #[derive(Debug, Clone)]
    pub struct ServerTunnelConfig {
        /// Tunnel name.
        pub name: String,
        /// Local forwarded port.
        pub port: u16,
        /// Startup destination file path.
        pub destination_path: String,
        /// Optional I2CP settings.
        pub i2cp: Option<I2cpOptions>,
    }

    /// Address-book configuration shared with the runtime owner in library
    /// builds used by I2PControl tests.
    #[derive(Debug, Clone, Default)]
    pub struct AddressBookConfig {
        /// Default hosts source.
        pub default: Option<String>,
        /// Additional subscription sources.
        pub subscriptions: Option<Vec<String>>,
    }
}

#[cfg(feature = "i2pcontrol")]
#[path = "address_book.rs"]
pub mod address_book;

#[cfg(feature = "i2pcontrol")]
pub mod i2pcontrol;

#[cfg(feature = "i2pcontrol")]
#[path = "tunnel/client.rs"]
pub mod tunnel_client;

#[cfg(feature = "i2pcontrol")]
#[path = "tunnel/server.rs"]
pub mod tunnel_server;
