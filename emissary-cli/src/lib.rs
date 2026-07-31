// Emissary CLI library root.
//
// This crate is primarily a binary, but this lib.rs re-exports modules
// needed for integration testing of the i2pcontrol subsystem.

#[cfg(feature = "i2pcontrol")]
pub mod config {
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
